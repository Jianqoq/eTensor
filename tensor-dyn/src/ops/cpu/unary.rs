use std::borrow::Borrow;
use std::panic::Location;

use crate::backend::Cpu;
use crate::ops::cpu::unary::ErrHandler::InvalidOutSize;
use crate::tensor_base::_Tensor;
use crate::THREAD_POOL;
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::{ParallelSlice, ParallelSliceMut};
use tensor_common::err_handler::ErrHandler;
use tensor_common::shape_utils::mt_intervals;
use tensor_iterator::iterator_traits::ParStridedIteratorSimdZip;
use tensor_iterator::TensorIterator;
use tensor_traits::tensor::TensorCreator;
use tensor_traits::tensor::{CommonBounds, TensorInfo, TensorLike};
use tensor_traits::{Neg, NormalUaryOps};
use tensor_types::dtype::TypeCommon;
use tensor_types::into_scalar::IntoScalar;
use tensor_types::type_promote::{Eval, NormalOut, NormalOutUnary};
use tensor_types::vectors::traits::*;
use threadpool::ThreadPool;

pub(crate) fn uary_fn_with_out_simd<A, O, K, F, F2>(
    inp: &_Tensor<A, Cpu>,
    f: F,
    f2: F2,
    out: Option<O>,
) -> anyhow::Result<_Tensor<K, Cpu>>
where
    A: CommonBounds,
    K: CommonBounds,
    O: Borrow<_Tensor<K, Cpu>>,
    F: Fn(A::Vec) -> K::Vec + Sync + Send,
    F2: Fn(A) -> K + Sync + Send,
{
    let mut ret = if let Some(out) = out {
        if out.borrow().size() * size_of::<K>() == inp.size() * size_of::<A>() {
            out.borrow().static_cast()?
        } else {
            return Err(InvalidOutSize(
                inp.size() * size_of::<A>(),
                out.borrow().size() * size_of::<K>(),
                Location::caller(),
            )
            .into());
        }
    } else {
        _Tensor::<K, Cpu>::empty(inp.shape())?
    };
    let ret_size = ret.size();
    if inp.parent().is_some() {
        ret.par_iter_mut_simd()
            .zip(inp.par_iter_simd())
            .for_each(|(a, b)| {
                *a = f2(b);
            });
        return Ok(ret);
    }
    let per_thread_len = ret.size() / rayon::current_num_threads();
    let per_thread_remain = per_thread_len % K::Vec::SIZE;
    let total_remain = rayon::current_num_threads() * per_thread_remain
        + (ret.size() % rayon::current_num_threads());
    let per_thread_real_len = per_thread_len - per_thread_remain;
    if per_thread_real_len > 0 {
        ret.as_raw_mut()
            .par_chunks_exact_mut(per_thread_real_len)
            .zip(inp.as_raw().par_chunks_exact(per_thread_real_len))
            .for_each(|(ret, lhs)| {
                assert_eq!(lhs.len() % A::Vec::SIZE, 0);
                assert_eq!(ret.len() % K::Vec::SIZE, 0);
                ret.chunks_exact_mut(A::Vec::SIZE)
                    .zip(lhs.chunks_exact(A::Vec::SIZE))
                    .for_each(|(ret, lhs)| {
                        let a = unsafe { A::Vec::from_ptr(lhs.as_ptr()) };
                        let res = f(a);
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                res.as_ptr(),
                                ret.as_mut_ptr(),
                                K::Vec::SIZE,
                            );
                        }
                    });
            });
    }
    if total_remain > 0 {
        ret.as_raw_mut()[ret_size - total_remain..]
            .iter_mut()
            .zip(inp.as_raw()[ret_size - total_remain..].iter())
            .for_each(|(a, &lhs)| {
                *a = f2(lhs);
            });
    }
    Ok(ret)
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

    fn floor(&self) -> anyhow::Result<_Tensor<NormalType<T>>> {
        uary_fn_with_out_simd(
            self,
            |x| x._floor(),
            |x| x._floor(),
            None::<_Tensor<NormalType<T>>>,
        )
    }

    fn floor_<U>(&self, out: U) -> anyhow::Result<Self::Output>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        uary_fn_with_out_simd(self, |x| x._floor(), |x| x._floor(), Some(out))
    }

    fn square(&self) -> anyhow::Result<_Tensor<NormalType<T>>> {
        uary_fn_with_out_simd(
            self,
            |x| x._square(),
            |x| x._square(),
            None::<_Tensor<NormalType<T>>>,
        )
    }

    fn square_<U>(&self, out: U) -> anyhow::Result<_Tensor<NormalType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        uary_fn_with_out_simd(self, |x| x._square(), |x| x._square(), Some(out))
    }

    fn abs(&self) -> anyhow::Result<Self> {
        uary_fn_with_out_simd(self, |x| x._abs(), |x| x._abs(), None::<Self::Output>)
    }

    fn abs_<U>(&self, out: U) -> anyhow::Result<Self>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        uary_fn_with_out_simd(self, |x| x._abs(), |x| x._abs(), Some(out))
    }

    fn ceil(&self) -> anyhow::Result<_Tensor<NormalType<T>>> {
        uary_fn_with_out_simd(
            self,
            |x| x._ceil(),
            |x| x._ceil(),
            None::<_Tensor<NormalType<T>>>,
        )
    }
    fn ceil_<U>(&self, out: U) -> anyhow::Result<_Tensor<NormalType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        uary_fn_with_out_simd(self, |x| x._ceil(), |x| x._ceil(), Some(out))
    }

    fn sign(&self) -> anyhow::Result<_Tensor<NormalType<T>>> {
        uary_fn_with_out_simd(
            self,
            |x| x._sign(),
            |x| x._sign(),
            None::<_Tensor<NormalType<T>>>,
        )
    }
    fn sign_<U>(&self, out: U) -> anyhow::Result<_Tensor<NormalType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        uary_fn_with_out_simd(self, |x| x._sign(), |x| x._sign(), Some(out))
    }
    fn clip(
        &self,
        min: NormalType<T>,
        max: NormalType<T>,
    ) -> anyhow::Result<_Tensor<NormalType<T>>> {
        let min_vec = T::Vec::splat(min);
        let max_vec = T::Vec::splat(max);
        uary_fn_with_out_simd(
            self,
            |x| x._clip(min_vec, max_vec),
            |x| <T as NormalOut<T>>::_clip(x, min, max),
            None::<_Tensor<NormalType<T>>>,
        )
    }
    fn clip_<U>(
        &self,
        min: NormalType<T>,
        max: NormalType<T>,
        out: U,
    ) -> anyhow::Result<_Tensor<NormalType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        let min_vec = T::Vec::splat(min);
        let max_vec = T::Vec::splat(max);
        uary_fn_with_out_simd(
            self,
            |x| x._clip(min_vec, max_vec),
            |x| <T as NormalOut<T>>::_clip(x, min, max),
            Some(out),
        )
    }
    fn round(&self) -> anyhow::Result<_Tensor<NormalType<T>>> {
        uary_fn_with_out_simd(
            self,
            |x| x._round(),
            |x| x._round(),
            None::<_Tensor<NormalType<T>>>,
        )
    }
    fn round_<U>(&self, out: U) -> anyhow::Result<_Tensor<NormalType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        uary_fn_with_out_simd(self, |x| x._round(), |x| x._round(), Some(out))
    }
}

type NegType<T> = <T as std::ops::Neg>::Output;

impl<T> Neg for _Tensor<T>
where
    T: std::ops::Neg<Output = T> + CommonBounds,
    NegType<T>: CommonBounds,
    _Tensor<NegType<T>>: TensorLike<NegType<T>>,
    T::Vec: std::ops::Neg<Output = T::Vec>,
{
    type Output = _Tensor<NegType<T>>;

    type InplaceOutput = _Tensor<NegType<T>>;

    type OutputMeta = NegType<T>;

    fn neg(&self) -> anyhow::Result<_Tensor<NegType<T>>> {
        uary_fn_with_out_simd(self, |x| -x, |x| -x, None::<_Tensor<NegType<T>>>)
    }

    fn neg_<U>(&self, out: U) -> anyhow::Result<_Tensor<NegType<T>>>
    where
        U: Borrow<Self::InplaceOutput>,
    {
        uary_fn_with_out_simd(self, |x| -x, |x| -x, Some(out))
    }
}

impl<T> _Tensor<T>
where
    T: CommonBounds + Eval,
    <T as Eval>::Output: CommonBounds,
    T::Vec: Eval<Output = <<T as Eval>::Output as TypeCommon>::Vec>,
{
    /// Checks for infinity (`inf`) values in the tensor.
    ///
    /// This method returns a new tensor where each element indicates whether the corresponding element
    /// in the input tensor is an infinity value (`+inf` or `-inf`). The output tensor will contain boolean-like values
    /// (1 for `inf`, 0 for non-`inf`).
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing a tensor of type `_Tensor<<T as Eval>::Output>`,
    /// where each element is either `1` (if the corresponding element is `inf`) or `0` (if it is not).
    pub fn is_inf(&self) -> anyhow::Result<_Tensor<<T as Eval>::Output>> {
        uary_fn_with_out_simd(
            self,
            |x| x._is_inf(),
            |x| x._is_inf(),
            None::<_Tensor<<T as Eval>::Output>>,
        )
    }

    /// Checks for `NaN` (Not-a-Number) values in the tensor.
    ///
    /// This method returns a new tensor where each element indicates whether the corresponding element
    /// in the input tensor is a `NaN` value. The output tensor will contain boolean-like values
    /// (1 for `NaN`, 0 for non-`NaN`).
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing a tensor of type `_Tensor<<T as Eval>::Output>`,
    /// where each element is either `1` (if the corresponding element is `NaN`) or `0` (if it is not).
    pub fn is_nan(&self) -> anyhow::Result<_Tensor<<T as Eval>::Output>> {
        uary_fn_with_out_simd(
            self,
            |x| x._is_nan(),
            |x| x._is_nan(),
            None::<_Tensor<<T as Eval>::Output>>,
        )
    }
}

impl<T> _Tensor<T>
where
    T: CommonBounds,
{
    /// Computes the cumulative sum of the elements along a specified axis.
    ///
    /// This method calculates the cumulative sum of the elements in the tensor along the given `axis`.
    /// The cumulative sum of an element at position `i` is the sum of all elements from the start of the axis
    /// up to and including position `i`. If no axis is specified, the cumulative sum is computed over a flattened
    /// version of the tensor.
    ///
    /// # Arguments
    ///
    /// * `axis` - An optional axis along which to compute the cumulative sum. If `None`, the tensor is flattened,
    ///   and the cumulative sum is computed over all elements.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing a new tensor with the cumulative sum computed along the specified axis.
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
                let mut res = _Tensor::<T, Cpu>::empty(vec![self.size() as i64])?;
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

    /// Computes the cumulative product of the elements along a specified axis.
    ///
    /// This method calculates the cumulative product of the elements in the tensor along the given `axis`.
    /// The cumulative product of an element at position `i` is the product of all elements from the start of the axis
    /// up to and including position `i`. If no axis is specified, the cumulative product is computed over a flattened
    /// version of the tensor.
    ///
    /// # Arguments
    ///
    /// * `axis` - An optional axis along which to compute the cumulative product. If `None`, the tensor is flattened,
    ///   and the cumulative product is computed over all elements.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing a new tensor with the cumulative product computed along the specified axis.
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
                let mut res = _Tensor::<T, Cpu>::empty(vec![self.size() as i64])?;
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
