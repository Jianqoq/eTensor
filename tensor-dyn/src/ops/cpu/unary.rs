use std::borrow::Borrow;

use crate::backend::Cpu;
use crate::tensor_base::_Tensor;
use crate::THREAD_POOL;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use rayon::slice::{ParallelSlice, ParallelSliceMut};
use tensor_common::err_handler::ErrHandler;
use tensor_common::shape_utils::mt_intervals;
use tensor_traits::tensor::TensorCreator;
use tensor_traits::tensor::{CommonBounds, TensorInfo, TensorLike};
use tensor_traits::{FloatUaryOps, Neg, NormalUaryOps};
use tensor_types::dtype::TypeCommon;
use tensor_types::into_scalar::IntoScalar;
use tensor_types::type_promote::{Eval, FloatOutBinary, FloatOutUnary, NormalOut};
use tensor_types::vectors::traits::*;
use threadpool::ThreadPool;

/// Applies a unary function to a tensor, returning a tensor with float type elements.
///
/// This function applies a unary function `f` to each element of the input tensor `inp` and
/// returns a new tensor with the result of the function. The result tensor has float type elements.
///
/// # Arguments
/// - `inp`: Input tensor.
/// - `f`: Unary function to apply to each element.
///
/// # Returns
/// `anyhow::Result<_Tensor<BFLOAT<A, A>>>`: A tensor with float type elements, with the result of applying `f`.

#[inline(always)]
pub fn uary_fn<A, F, O>(inp: &_Tensor<A>, f: F) -> anyhow::Result<_Tensor<O>>
where
    A: CommonBounds,
    O: CommonBounds,
    F: Fn(A) -> O + Sync + Send,
{
    let ret: _Tensor<O>;
    ret = _Tensor::<O, Cpu>::empty(inp.shape())?;
    let new_f = &f;
    let remain = inp.size() % 8;
    let exect_size = inp.size() - remain;
    ret.as_raw_mut()
        .par_chunks_exact_mut(8)
        .zip(inp.as_raw().par_chunks_exact(8))
        .for_each(|(a, b)| {
            a[0] = new_f(b[0]);
            a[1] = new_f(b[1]);
            a[2] = new_f(b[2]);
            a[3] = new_f(b[3]);
            a[4] = new_f(b[4]);
            a[5] = new_f(b[5]);
            a[6] = new_f(b[6]);
            a[7] = new_f(b[7]);
        });
    ret.as_raw_mut()[exect_size..]
        .iter_mut()
        .zip(inp.as_raw()[exect_size..].iter())
        .for_each(|(a, &b)| {
            *a = f(b);
        });
    Ok(ret)
}

#[inline(always)]
pub fn uary_fn_simd<A, F, O, F2>(inp: &_Tensor<A>, f: F, f2: F2) -> anyhow::Result<_Tensor<O>>
where
    A: CommonBounds,
    O: CommonBounds,
    F: Fn(<A as TypeCommon>::Vec) -> <O as TypeCommon>::Vec + Sync + Send,
    F2: Fn(A) -> O + Sync + Send,
{
    let ret: _Tensor<O>;
    ret = _Tensor::<O, Cpu>::empty(inp.shape())?;
    if inp.parent().is_some() {
        ret.par_iter_mut_simd()
            .zip(inp.par_iter_simd())
            .for_each(|(a, b)| {
                *a = f2(b);
            });
        return Ok(ret);
    }
    let per_thread_len = ret.size() / rayon::current_num_threads();
    let per_thread_remain = per_thread_len % <O as TypeCommon>::Vec::SIZE;
    let total_remain = rayon::current_num_threads() * per_thread_remain
        + (ret.size() % rayon::current_num_threads());
    let per_thread_real_len = per_thread_len - per_thread_remain;
    if per_thread_real_len > 0 {
        ret.as_raw_mut()
            .par_chunks_exact_mut(per_thread_real_len)
            .zip(inp.as_raw().par_chunks_exact(per_thread_real_len))
            .for_each(|(ret, lhs)| {
                assert_eq!(lhs.len() % <A as TypeCommon>::Vec::SIZE, 0);
                assert_eq!(ret.len() % <O as TypeCommon>::Vec::SIZE, 0);
                ret.chunks_exact_mut(<A as TypeCommon>::Vec::SIZE)
                    .zip(lhs.chunks_exact(<A as TypeCommon>::Vec::SIZE))
                    .for_each(|(ret, lhs)| {
                        let a = unsafe { <A as TypeCommon>::Vec::from_ptr(lhs.as_ptr()) };
                        let res = f(a);
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                res.as_ptr(),
                                ret.as_mut_ptr(),
                                <O as TypeCommon>::Vec::SIZE,
                            );
                        }
                    });
            });
    }
    if total_remain > 0 {
        ret.as_raw_mut()[ret.size() - total_remain..]
            .iter_mut()
            .zip(inp.as_raw()[ret.size() - total_remain..].iter())
            .for_each(|(a, &lhs)| {
                *a = f2(lhs);
            });
    }
    Ok(ret)
}

/// Applies a unary function to a tensor with a specified output tensor.
///
/// Similar to `uary_fn_float`, but allows specifying an output tensor. The output tensor's memory
/// is reused if it matches the required size and reference count conditions; otherwise, a new tensor is created.
///
/// # Arguments
/// - `inp`: Input tensor.
/// - `f`: Unary function to apply to each element.
///
/// # Returns
/// `anyhow::Result<_Tensor<BFLOAT<A, A>>>`: A tensor with float type elements, with the result of applying `f`.
pub fn uary_fn_with_out<A, O, K, Q, F>(
    inp: &_Tensor<A, Cpu>,
    f: F,
    out: O,
) -> anyhow::Result<_Tensor<K, Cpu>>
where
    A: CommonBounds,
    O: Borrow<_Tensor<Q, Cpu>>,
    K: CommonBounds,
    Q: CommonBounds,
    F: Fn(A) -> K + Sync + Send,
{
    let ret_size: usize = inp.size();
    let ret = if out.borrow().size() * size_of::<Q>() != ret_size * size_of::<K>() {
        _Tensor::<K, Cpu>::empty(inp.shape())?
    } else {
        _Tensor::<K, Cpu>::empty(inp.shape())?
    };
    ret.as_raw_mut()
        .par_iter_mut()
        .zip(inp.as_raw().par_iter())
        .for_each(|(a, &b)| {
            *a = f(b);
        });
    Ok(ret)
}

#[cfg(feature = "simd")]
pub(crate) fn uary_fn_with_out_simd<A, O, K, Q, F, F2>(
    inp: &_Tensor<A, Cpu>,
    f: F,
    f2: F2,
    out: O,
) -> anyhow::Result<_Tensor<K, Cpu>>
where
    A: CommonBounds,
    O: Borrow<_Tensor<Q, Cpu>>,
    K: CommonBounds,
    Q: CommonBounds,
    F: Fn(<A as TypeCommon>::Vec) -> <K as TypeCommon>::Vec + Sync + Send,
    F2: Fn(A) -> K + Sync + Send,
{
    let ret_size: usize = inp.size();
    let ret = if out.borrow().size() * size_of::<Q>() != ret_size * size_of::<K>() {
        _Tensor::<K, Cpu>::empty(inp.shape())?
    } else {
        _Tensor::<K, Cpu>::empty(inp.shape())?
    };
    let remain = inp.size() % <A as TypeCommon>::Vec::SIZE;
    let exect_size = inp.size() - remain;
    ret.as_raw_mut()
        .par_chunks_exact_mut(<A as TypeCommon>::Vec::SIZE)
        .zip(inp.as_raw().par_chunks_exact(<A as TypeCommon>::Vec::SIZE))
        .for_each(|(a, b)| {
            let b_ptr = b.as_ptr();
            let inp = unsafe { A::Vec::from_ptr(b_ptr) };
            let res = f(inp);
            let res_ptr = res.as_ptr() as *mut K;
            unsafe {
                std::ptr::copy_nonoverlapping(
                    res_ptr,
                    a.as_mut_ptr(),
                    <A as TypeCommon>::Vec::SIZE,
                );
            }
        });
    ret.as_raw_mut()[exect_size..]
        .iter_mut()
        .zip(inp.as_raw()[exect_size..].iter())
        .for_each(|(a, &b)| {
            *a = f2(b);
        });
    Ok(ret)
}

pub(crate) type FloatUnaryType<T> = <T as FloatOutUnary>::Output;
pub(crate) type FloatBinaryType<T> = <T as FloatOutBinary>::Output;

impl<T> FloatUaryOps for _Tensor<T>
where
    T: FloatOutUnary<Base = FloatUnaryType<T>> + CommonBounds,
    FloatUnaryType<T>: CommonBounds,
    f64: IntoScalar<<T as FloatOutUnary>::Output>,
    T::Vec:
        FloatOutUnary<Output = <FloatUnaryType<T> as TypeCommon>::Vec, Base = FloatUnaryType<T>>,
    <FloatUnaryType<T> as TypeCommon>::Vec: Send + Copy + Sync,
{
    type Output = _Tensor<FloatUnaryType<T>>;

    type InplaceOutput = _Tensor<FloatUnaryType<T>>;

    type OutputMeta = FloatUnaryType<T>;

    #[cfg_attr(feature = "track_caller", track_caller)]
    fn sin(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._sin(), |x| x._sin());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._sin());
        ret
    }

    #[cfg_attr(feature = "track_caller", track_caller)]
    fn cos(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._cos(), |x| x._cos());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._cos());
        ret
    }

    #[cfg_attr(feature = "track_caller", track_caller)]
    fn tan(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._tan(), |x| x._tan());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._tan());
        ret
    }

    #[cfg_attr(feature = "track_caller", track_caller)]
    fn asin(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._asin(), |x| x._asin());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._asin());
        ret
    }

    /// calculate `acos` for each element of the tensor
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn acos(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._acos(), |x| x._acos());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._acos());
        ret
    }

    /// calculate `atan` for each element of the tensor
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn atan(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._atan(), |x| x._atan());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._atan());
        ret
    }

    /// calculate `sinh` for each element of the tensor
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn sinh(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._sinh(), |x| x._sinh());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._sinh());
        ret
    }

    /// calculate `cosh` for each element of the tensor
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn cosh(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._cosh(), |x| x._cosh());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._cosh());
        ret
    }

    /// calculate `erf` for each element of the tensor
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn erf(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._erf(), |x| x._erf());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._erf());
        ret
    }

    /// calculate `tanh` for each element of the tensor
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn tanh(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._tanh(), |x| x._tanh());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._tanh());
        ret
    }

    /// calculate `asinh` for each element of the tensor
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn asinh(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._asinh(), |x| x._asinh());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._asinh());
        ret
    }

    /// calculate `acosh` for each element of the tensor
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn acosh(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._acosh(), |x| x._acosh());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._acosh());
        ret
    }

    /// calculate `atanh` for each element of the tensor
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn atanh(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._atanh(), |x| x._atanh());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._atanh());
        ret
    }

    /// calculate `sin` for each element of the tensor and store the result in the output tensor`
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn sin_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<_Tensor<FloatUnaryType<T>>>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._sin(), |x| x._sin(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._sin(), out);
        ret
    }

    /// calculate `cos` for each element of the tensor and store the result in the output tensor`
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn cos_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._cos(), |x| x._cos(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._cos(), out);
        ret
    }

    /// calculate `tan` for each element of the tensor and store the result in the output tensor`
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn tan_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._tan(), |x| x._tan(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._tan(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn asin_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._asin(), |x| x._asin(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._asin(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn acos_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._acos(), |x| x._acos(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._acos(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn atan_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._atan(), |x| x._atan(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._atan(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn sinh_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._sinh(), |x| x._sinh(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._sinh(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn cosh_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._cosh(), |x| x._cosh(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._cosh(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn tanh_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._tanh(), |x| x._tanh(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._tanh(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn asinh_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._asinh(), |x| x._asinh(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._asinh(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn acosh_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._acosh(), |x| x._acosh(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._acosh(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn atanh_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._atanh(), |x| x._atanh(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._atanh(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn exp(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._exp(), |x| x._exp());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._exp());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn exp_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._exp(), |x| x._exp(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._exp(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn exp2(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._exp2(), |x| x._exp2());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._exp2());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn exp2_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._exp2(), |x| x._exp2(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._exp2(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn sqrt(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._sqrt(), |x| x._sqrt());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._sqrt());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn sqrt_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._sqrt(), |x| x._sqrt(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._sqrt(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn recip(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._recip(), |x| x._recip());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._recip());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn recip_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._recip(), |x| x._recip(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._recip(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn ln(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._ln(), |x| x._ln());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._ln());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn ln_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._ln(), |x| x._ln(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._ln(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn log2(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._log2(), |x| x._log2());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._log2());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn log2_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._log2(), |x| x._log2(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._log2(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn log10(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._log10(), |x| x._log10());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._log10());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn log10_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._log10(), |x| x._log10(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._log10(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn celu(&self, alpha: FloatUnaryType<T>) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._celu(alpha), |x| x._celu(alpha));
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._celu(alpha));
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn celu_<U>(
        &self,
        alpha: FloatUnaryType<T>,
        out: U,
    ) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._celu(alpha), |x| x._celu(alpha), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._celu(alpha), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn sigmoid(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._sigmoid(), |x| x._sigmoid());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._sigmoid());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn sigmoid_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._sigmoid(), |x| x._sigmoid(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._sigmoid(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn elu(&self, alpha: FloatUnaryType<T>) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._elu(alpha), |x| x._elu(alpha));
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._elu(alpha));
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn elu_<U>(
        &self,
        alpha: FloatUnaryType<T>,
        out: U,
    ) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._elu(alpha), |x| x._elu(alpha), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._elu(alpha), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn leaky_relu(&self, alpha: FloatUnaryType<T>) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._leaky_relu(alpha), |x| x._leaky_relu(alpha));
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._leaky_relu(alpha));
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn leaky_relu_<U>(
        &self,
        alpha: FloatUnaryType<T>,
        out: U,
    ) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(
            self,
            |x| x._leaky_relu(alpha),
            |x| x._leaky_relu(alpha),
            out,
        );
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._leaky_relu(alpha), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn gelu(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._gelu(), |x| x._gelu());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._gelu());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn gelu_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._gelu(), |x| x._gelu(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._gelu(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn selu(
        &self,
        alpha: Option<FloatUnaryType<T>>,
        gamma: Option<FloatUnaryType<T>>,
    ) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        let alpha = alpha.unwrap_or(1.6732632423543772848170429916717.into_scalar());
        let gamma = gamma.unwrap_or(1.0507009873554804934193349852946.into_scalar());
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(
            self,
            |x| x._selu(alpha, gamma),
            move |x| x._selu(alpha, gamma),
        );
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._selu(alpha, gamma));
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn selu_<U>(
        &self,
        alpha: Option<FloatUnaryType<T>>,
        gamma: Option<FloatUnaryType<T>>,
        out: U,
    ) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        let alpha = alpha.unwrap_or(1.67326319217681884765625.into_scalar());
        let gamma = gamma.unwrap_or(1.05070102214813232421875.into_scalar());
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(
            self,
            |x| x._selu(alpha, gamma),
            |x| x._selu(alpha, gamma),
            out,
        );
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._selu(alpha, gamma), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn hard_sigmoid(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._hard_sigmoid(), |x| x._hard_sigmoid());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._hard_sigmoid());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn fast_hard_sigmoid(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._fast_hard_sigmoid(), |x| x._fast_hard_sigmoid());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._hard_sigmoid());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn hard_sigmoid_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._hard_sigmoid(), |x| x._hard_sigmoid(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._hard_sigmoid(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn hard_swish(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._hard_swish(), |x| x._hard_swish());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._hard_swish());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn hard_swish_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._hard_swish(), |x| x._hard_swish(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._hard_swish(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn relu6(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._relu6(), |x| x._relu6());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._relu6());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn relu6_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._relu6(), |x| x._relu6(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._relu6(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn softplus(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._softplus(), |x| x._softplus());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._softplus());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn softplus_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._softplus(), |x| x._softplus(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._softplus(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn softsign(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._softsign(), |x| x._softsign());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._softsign());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn softsign_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._softsign(), |x| x._softsign(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._softsign(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn mish(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._mish(), |x| x._mish());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._mish());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn mish_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._mish(), |x| x._mish(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._mish(), out);
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn relu(&self) -> anyhow::Result<_Tensor<FloatUnaryType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._relu(), |x| x._relu());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._mish());
        ret
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn relu_<U>(&self, out: U) -> anyhow::Result<_Tensor<FloatUnaryType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._relu(), |x| x._relu(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._mish(), out);
        ret
    }
}

pub(crate) type NormalType<T> = <T as NormalOut>::Output;

impl<T> NormalUaryOps for _Tensor<T>
where
    T: NormalOut<Output = T> + CommonBounds + IntoScalar<T>,
    NormalType<T>: CommonBounds,
    _Tensor<NormalType<T>>: TensorLike<NormalType<T>>,
    T::Vec: NormalOut<Output = T::Vec>,
{
    type Output = _Tensor<NormalType<T>>;

    type InplaceOutput = _Tensor<NormalType<T>>;

    type OutputMeta = NormalType<T>;
    /// calculate `floor` for each element of the tensor
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn floor(&self) -> anyhow::Result<_Tensor<NormalType<T>>> {
        #[cfg(feature = "simd")]
        let ret = uary_fn_simd(self, |x| x._floor(), |x| x._floor());
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn(self, |x| x._floor());
        ret
    }

    #[cfg_attr(feature = "track_caller", track_caller)]
    fn floor_<U>(&self, out: U) -> anyhow::Result<Self::Output>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        #[cfg(feature = "simd")]
        let ret = uary_fn_with_out_simd(self, |x| x._floor(), |x| x._floor(), out);
        #[cfg(not(feature = "simd"))]
        let ret = uary_fn_with_out(self, |x| x._floor(), out);
        ret
    }

    #[cfg_attr(feature = "track_caller", track_caller)]
    fn square(&self) -> anyhow::Result<_Tensor<NormalType<T>>> {
        uary_fn(self, |x| x._square())
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn square_<U>(&self, out: U) -> anyhow::Result<_Tensor<NormalType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        uary_fn_with_out(self, |x| x._square(), out)
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn abs(&self) -> anyhow::Result<Self> {
        uary_fn(self, |x| x._abs())
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn abs_<U>(&self, out: U) -> anyhow::Result<Self>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        uary_fn_with_out(self, |x| x._abs(), out)
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn ceil(&self) -> anyhow::Result<_Tensor<NormalType<T>>> {
        uary_fn(self, |x| x._ceil())
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn ceil_<U>(&self, out: U) -> anyhow::Result<_Tensor<NormalType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        uary_fn_with_out(self, |x| x._ceil(), out)
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn sign(&self) -> anyhow::Result<_Tensor<NormalType<T>>> {
        uary_fn(self, |x| x._sign())
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn sign_<U>(&self, out: U) -> anyhow::Result<_Tensor<NormalType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        uary_fn_with_out(self, |x| x._sign(), out)
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn clip(
        &self,
        min: NormalType<T>,
        max: NormalType<T>,
    ) -> anyhow::Result<_Tensor<NormalType<T>>> {
        uary_fn(self, |x| x._clip(min, max))
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn clip_<U>(
        &self,
        min: NormalType<T>,
        max: NormalType<T>,
        out: U,
    ) -> anyhow::Result<_Tensor<NormalType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        uary_fn_with_out(self, |x| x._clip(min, max), out)
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn round(&self) -> anyhow::Result<_Tensor<NormalType<T>>> {
        uary_fn(self, |x| x._round())
    }
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn round_<U>(&self, out: U) -> anyhow::Result<_Tensor<NormalType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        uary_fn_with_out(self, |x| x._round(), out)
    }
}

type NegType<T> = <T as std::ops::Neg>::Output;

impl<T> Neg for _Tensor<T>
where
    T: std::ops::Neg + CommonBounds,
    NegType<T>: CommonBounds,
    _Tensor<NegType<T>>: TensorLike<NegType<T>>,
{
    type Output = _Tensor<NegType<T>>;

    type InplaceOutput = _Tensor<NegType<T>>;

    type OutputMeta = NegType<T>;

    #[allow(unused)]
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn neg(&self) -> anyhow::Result<_Tensor<NegType<T>>> {
        uary_fn(self, |x| -x)
    }

    #[allow(unused)]
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn neg_<U>(&self, out: U) -> anyhow::Result<_Tensor<NegType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        uary_fn_with_out(self, |x| -x, out)
    }
}

impl<T> _Tensor<T>
where
    T: CommonBounds + Eval,
    <T as Eval>::Output: CommonBounds,
{
    pub fn is_inf(&self) -> anyhow::Result<_Tensor<<T as Eval>::Output>> {
        uary_fn(self, |x| x._is_inf())
    }
    pub fn is_nan(&self) -> anyhow::Result<_Tensor<<T as Eval>::Output>> {
        uary_fn(self, |x| x._is_nan())
    }
}

impl<T> _Tensor<T>
where
    T: CommonBounds,
{
    #[allow(unused)]
    #[cfg_attr(feature = "track_caller", track_caller)]
    pub fn cumsum(&self, axis: Option<i64>) -> anyhow::Result<Self>
    where
        T: NormalOut<T, Output = T>,
    {
        match axis {
            Some(axis) => {
                let mut _axis = axis;
                ErrHandler::check_index_in_range_mut(self.ndim(), &mut _axis)?;
                let stride = self.strides()[_axis as usize];
                let inner_loop = self.shape()[_axis as usize] as usize;
                let outer_loop = self.size() / inner_loop;
                let mut shape = self.shape().to_vec();
                shape.iter_mut().for_each(|x| {
                    *x -= 1;
                });
                shape.swap(_axis as usize, self.shape().len() - 1);
                let mut strides = self.strides().to_vec();
                strides.swap(_axis as usize, self.strides().len() - 1);
                let res = self.empty_like()?;
                let res_stride = res.strides()[_axis as usize];
                let mut res_strides = res.strides().to_vec();
                res_strides.swap(_axis as usize, res.strides().len() - 1);
                THREAD_POOL.with_borrow_mut(|pool: &mut ThreadPool| {
                    let num_threads;
                    if outer_loop < pool.max_count() {
                        num_threads = 1;
                    } else {
                        num_threads = 1;
                    }
                    let mut intervals = mt_intervals(outer_loop, num_threads);

                    let mut prgs = Vec::with_capacity(num_threads);
                    let mut ptrs = Vec::with_capacity(num_threads);
                    let mut res_ptrs = Vec::with_capacity(num_threads);
                    let mut shapes = Vec::with_capacity(num_threads);
                    let mut __res_strides = Vec::with_capacity(num_threads);
                    let mut __inp_strides = Vec::with_capacity(num_threads);
                    let mut amount = 0i64;
                    for i in 0..num_threads {
                        let (start, end) = intervals[i];
                        let mut amount_cpy = amount;
                        let mut prg_tmp = vec![0; self.shape().len() - 1];
                        let mut ptr_tmp = self.ptr();
                        let mut res_ptr_tmp = res.ptr();
                        for j in (0..(self.shape().len() as i64) - 1).rev() {
                            prg_tmp[j as usize] = amount_cpy % (shape[j as usize] + 1);
                            amount_cpy /= self.shape()[j as usize];
                            let inp_gap = prg_tmp[j as usize] * strides[j as usize];
                            let res_gap = prg_tmp[j as usize] * res_strides[j as usize];
                            ptr_tmp.offset(inp_gap);
                            res_ptr_tmp.offset(res_gap);
                        }
                        amount += ((end - start) as i64) * (shape[shape.len() - 1] + 1);
                        prgs.push(prg_tmp);
                        ptrs.push(ptr_tmp);
                        res_ptrs.push(res_ptr_tmp);
                        shapes.push(shape.clone());
                        __res_strides.push(res_strides.clone());
                        __inp_strides.push(strides.clone());
                    }
                    for _ in 0..num_threads {
                        let (start, end) = intervals.pop().unwrap();
                        let mut prg = prgs.pop().unwrap();
                        let mut ptr = ptrs.pop().unwrap();
                        let mut res_ptr = res_ptrs.pop().unwrap();
                        let current_size = end - start;
                        let __shape = shapes.pop().unwrap();
                        let __res_strides = __res_strides.pop().unwrap();
                        let __strides = __inp_strides.pop().unwrap();
                        pool.execute(move || {
                            for _ in 0..current_size {
                                let mut tmp = T::ZERO;
                                for i in 0..inner_loop as i64 {
                                    tmp = tmp._add(ptr[i * stride]);
                                    res_ptr.modify(i * res_stride, tmp);
                                }
                                for j in (0..(__shape.len() as i64) - 1).rev() {
                                    let j = j as usize;
                                    if prg[j] < __shape[j] {
                                        prg[j] += 1;
                                        res_ptr.offset(__res_strides[j]);
                                        ptr.offset(__strides[j]);
                                        break;
                                    } else {
                                        prg[j] = 0;
                                        res_ptr.offset(-__shape[j] * __res_strides[j]);
                                        ptr.offset(-__shape[j] * __strides[j]);
                                    }
                                }
                            }
                        });
                    }
                });
                Ok(res)
            }
            None => {
                let res = _Tensor::<T, Cpu>::empty(vec![self.size() as i64])?;
                let mut tmp = T::ZERO;
                if self.is_contiguous() {
                    let raw = self.as_raw();
                    let res_raw = res.as_raw_mut();
                    for i in 0..self.size() {
                        tmp = tmp._add(raw[i]);
                        res_raw[i] = tmp;
                    }
                    Ok(res)
                } else {
                    let new_self = self.contiguous()?;
                    let raw = new_self.as_raw();
                    let mut tmp = T::ZERO;
                    let res_raw = res.as_raw_mut();
                    for i in 0..self.size() {
                        tmp = tmp._add(raw[i]);
                        res_raw[i] = tmp;
                    }
                    Ok(res)
                }
            }
        }
    }
    #[allow(unused)]
    #[cfg_attr(feature = "track_caller", track_caller)]
    pub fn cumprod(&self, axis: Option<i64>) -> anyhow::Result<Self>
    where
        T: NormalOut<T, Output = T>,
    {
        match axis {
            Some(axis) => {
                let mut _axis = axis;
                ErrHandler::check_index_in_range_mut(self.ndim(), &mut _axis)?;
                let stride = self.strides()[_axis as usize];
                let inner_loop = self.shape()[_axis as usize] as usize;
                let outer_loop = self.size() / inner_loop;
                let mut shape = self.shape().to_vec();
                shape.iter_mut().for_each(|x| {
                    *x -= 1;
                });
                shape.swap(_axis as usize, self.shape().len() - 1);
                let mut strides = self.strides().to_vec();
                strides.swap(_axis as usize, self.strides().len() - 1);
                let res = self.empty_like()?;
                let res_stride = res.strides()[_axis as usize];
                let mut res_strides = res.strides().to_vec();
                res_strides.swap(_axis as usize, res.strides().len() - 1);
                THREAD_POOL.with_borrow_mut(|pool: &mut ThreadPool| {
                    let num_threads;
                    if outer_loop < pool.max_count() {
                        num_threads = outer_loop;
                    } else {
                        num_threads = pool.max_count();
                    }
                    let mut intervals = mt_intervals(outer_loop, num_threads);

                    let mut prgs = Vec::with_capacity(num_threads);
                    let mut ptrs = Vec::with_capacity(num_threads);
                    let mut res_ptrs = Vec::with_capacity(num_threads);
                    let mut shapes = Vec::with_capacity(num_threads);
                    let mut __res_strides = Vec::with_capacity(num_threads);
                    let mut __inp_strides = Vec::with_capacity(num_threads);
                    let mut amount = 0i64;
                    for i in 0..num_threads {
                        let (start, end) = intervals[i];
                        let mut amount_cpy = amount;
                        let mut prg_tmp = vec![0; self.shape().len() - 1];
                        let mut ptr_tmp = self.ptr();
                        let mut res_ptr_tmp = res.ptr();
                        for j in (0..(self.shape().len() as i64) - 1).rev() {
                            prg_tmp[j as usize] = amount_cpy % (shape[j as usize] + 1);
                            amount_cpy /= self.shape()[j as usize];
                            let inp_gap = prg_tmp[j as usize] * strides[j as usize];
                            let res_gap = prg_tmp[j as usize] * res_strides[j as usize];
                            ptr_tmp.offset(inp_gap);
                            res_ptr_tmp.offset(res_gap);
                        }
                        amount += ((end - start) as i64) * (shape[shape.len() - 1] + 1);
                        prgs.push(prg_tmp);
                        ptrs.push(ptr_tmp);
                        res_ptrs.push(res_ptr_tmp);
                        shapes.push(shape.clone());
                        __res_strides.push(res_strides.clone());
                        __inp_strides.push(strides.clone());
                    }
                    for _ in 0..num_threads {
                        let (start, end) = intervals.pop().unwrap();
                        let mut prg = prgs.pop().unwrap();
                        let mut ptr = ptrs.pop().unwrap();
                        let mut res_ptr = res_ptrs.pop().unwrap();
                        let current_size = end - start;
                        let __shape = shapes.pop().unwrap();
                        let __res_strides = __res_strides.pop().unwrap();
                        let __strides = __inp_strides.pop().unwrap();
                        pool.execute(move || {
                            for _ in 0..current_size {
                                let mut tmp = T::ONE;
                                for i in 0..inner_loop as i64 {
                                    tmp = tmp._mul(ptr[i * stride]);
                                    res_ptr.modify(i * res_stride, tmp);
                                }
                                for j in (0..(__shape.len() as i64) - 1).rev() {
                                    let j = j as usize;
                                    if prg[j] < __shape[j] {
                                        prg[j] += 1;
                                        res_ptr.offset(__res_strides[j]);
                                        ptr.offset(__strides[j]);
                                        break;
                                    } else {
                                        prg[j] = 0;
                                        res_ptr.offset(-__shape[j] * __res_strides[j]);
                                        ptr.offset(-__shape[j] * __strides[j]);
                                    }
                                }
                            }
                        });
                    }
                });
                Ok(res)
            }
            None => {
                let res = _Tensor::<T, Cpu>::empty(vec![self.size() as i64])?;
                let mut tmp = T::ONE;
                if self.is_contiguous() {
                    let raw = self.as_raw();
                    let res_raw = res.as_raw_mut();
                    for i in 0..self.size() {
                        tmp = tmp._mul(raw[i]);
                        res_raw[i] = tmp;
                    }
                    Ok(res)
                } else {
                    let new_self = self.contiguous()?;
                    let raw = new_self.as_raw();
                    let mut tmp = T::ONE;
                    let res_raw = res.as_raw_mut();
                    for i in 0..self.size() {
                        tmp = tmp._mul(raw[i]);
                        res_raw[i] = tmp;
                    }
                    Ok(res)
                }
            }
        }
    }
}
