use rayon::{
    iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator},
    slice::ParallelSliceMut,
};
use std::ops::{Mul, Sub};
use tensor_common::error::base::TensorError;
use tensor_traits::{CommonBounds, TensorCreator, TensorLike, WindowOps};
use tensor_types::{
    dtype::{FloatConst, TypeCommon},
    into_scalar::IntoScalar,
    traits::VecTrait,
    type_promote::{FloatOutBinary, FloatOutUnary, NormalOut},
};

use crate::{tensor_base::_Tensor, Cpu};

pub(crate) type Simd<T> = <<T as FloatOutBinary>::Output as TypeCommon>::Vec;
type FBO<T> = <T as FloatOutBinary>::Output;

impl<T, const DEVICE: usize> WindowOps for _Tensor<T, Cpu, DEVICE>
where
    f64: IntoScalar<FBO<T>>,
    T: CommonBounds + FloatOutBinary,
    FBO<T>: CommonBounds
        + FloatOutUnary<Output = FBO<T>>
        + Mul<Output = FBO<T>>
        + Sub<Output = FBO<T>>
        + FloatConst,
    FBO<T>: std::ops::Neg<Output = FBO<T>>,
    FBO<T>: NormalOut<FBO<T>, Output = FBO<T>> + FloatOutBinary<FBO<T>, Output = FBO<T>>,
    Simd<T>: NormalOut<Simd<T>, Output = Simd<T>>
        + FloatOutBinary<Simd<T>, Output = Simd<T>>
        + FloatOutUnary<Output = Simd<T>>,
    usize: IntoScalar<FBO<T>>,
    i64: IntoScalar<T>,
{
    type Output = _Tensor<FBO<T>, Cpu, DEVICE>;
    type Meta = T;

    #[cfg_attr(feature = "track_caller", track_caller)]
    fn hamming_window(window_length: i64, periodic: bool) -> Result<Self::Output, TensorError> {
        __hamming_window(
            window_length,
            (0.54).into_scalar(),
            (0.46).into_scalar(),
            periodic,
        )
    }

    #[cfg_attr(feature = "track_caller", track_caller)]
    fn hann_window(window_length: i64, periodic: bool) -> Result<Self::Output, TensorError> {
        __hamming_window(
            window_length,
            (0.5).into_scalar(),
            (0.5).into_scalar(),
            periodic,
        )
    }

    #[cfg_attr(feature = "track_caller", track_caller)]
    fn blackman_window(window_length: i64, periodic: bool) -> Result<Self::Output, TensorError>
    where
        Self::Meta: FloatConst,
        i64: IntoScalar<<Self::Meta as FloatOutBinary>::Output>,
    {
        let a0: <T as FloatOutBinary>::Output = (0.42).into_scalar();
        let a1: <T as FloatOutBinary>::Output = (0.5).into_scalar();
        let a2: <T as FloatOutBinary>::Output = (0.08).into_scalar();
        let length_usize = if periodic {
            window_length
        } else {
            window_length - 1
        };
        let length: <T as FloatOutBinary>::Output = length_usize.into_scalar();
        let mut ret =
            _Tensor::<<T as FloatOutBinary>::Output, Cpu, DEVICE>::empty(&[length_usize])?;
        ret.as_raw_mut()
            .par_iter_mut()
            .enumerate()
            .for_each(|(idx, x)| {
                let idx: <T as FloatOutBinary>::Output = idx.into_scalar();
                let a = a1._mul(T::TWOPI._mul(idx)._div(length)._cos());
                let b = a2._mul(T::FOURPI._mul(idx)._div(length)._cos());
                *x = a0._sub(a)._add(b);
            });
        Ok(ret)
    }
}

#[cfg_attr(feature = "track_caller", track_caller)]
fn __hamming_window<T, const DEVICE: usize>(
    window_length: i64,
    alpha: FBO<T>,
    beta: FBO<T>,
    periodic: bool,
) -> Result<_Tensor<FBO<T>, Cpu, DEVICE>, TensorError>
where
    f64: IntoScalar<FBO<T>>,
    T: CommonBounds + FloatOutBinary,
    FBO<T>: CommonBounds
        + FloatOutUnary<Output = FBO<T>>
        + Mul<Output = FBO<T>>
        + Sub<Output = FBO<T>>
        + FloatConst,
    FBO<T>: std::ops::Neg<Output = FBO<T>>,
    FBO<T>: NormalOut<FBO<T>, Output = FBO<T>> + FloatOutBinary<FBO<T>, Output = FBO<T>>,
    Simd<T>: NormalOut<Simd<T>, Output = Simd<T>>
        + FloatOutBinary<Simd<T>, Output = Simd<T>>
        + FloatOutUnary<Output = Simd<T>>,
    usize: IntoScalar<FBO<T>>,
    i64: IntoScalar<T>,
{
    let length_usize = (if periodic {
        window_length
    } else {
        window_length - 1
    }) as usize;
    let length: FBO<T> = length_usize.into_scalar();
    let mut ret = _Tensor::<FBO<T>, Cpu, DEVICE>::empty(&[length_usize as i64])?;
    let mut chunk_exact = ret.as_raw_mut().par_chunks_exact_mut(Simd::<T>::SIZE);
    let two_pi = Simd::<T>::splat(FBO::<T>::TWOPI);
    let length_vec = Simd::<T>::splat(length);
    let alpha_vec = Simd::<T>::splat(alpha);
    let beta_vec = Simd::<T>::splat(-beta);
    let remainder = chunk_exact.remainder();
    remainder.iter_mut().enumerate().for_each(|(idx, x)| {
        let idx: FBO<T> = idx.into_scalar();
        *x = idx
            ._mul(FBO::<T>::TWOPI._div(length))
            ._cos()
            ._mul_add(-beta, alpha);
    });
    chunk_exact.enumerate().for_each(|(x, vec)| {
        let idx = x * Simd::<T>::SIZE;
        let mut idxes = Simd::<T>::splat(FBO::<T>::ZERO);
        for i in 0..Simd::<T>::SIZE {
            idxes[i] = (idx + i).into_scalar();
        }
        let ptr = vec as *mut _ as *mut Simd<T>;

        let res = tensor_types::traits::VecTrait::mul_add(
            idxes._mul(two_pi._div(length_vec))._cos(),
            beta_vec,
            alpha_vec,
        );
        unsafe {
            ptr.write_unaligned(res);
        }
    });
    Ok(ret)
}
