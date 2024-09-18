use crate::backend::Cpu;
use crate::tensor_base::_Tensor;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use tensor_common::shape_utils::predict_broadcast_shape;
use tensor_traits::tensor::CommonBounds;
use tensor_traits::tensor::TensorCreator;
use tensor_traits::tensor::TensorInfo;
use tensor_traits::tensor::TensorLike;

#[cfg(feature = "simd")]
use tensor_types::dtype::TypeCommon;

/// binary function that takes two tensors and a kernel function and returns a new tensor
///
/// there are three cases:
///
/// - if one of the tensors has only one element, it will directly use the scalar value to perform the operation
///
/// - if both tensors have the same shape and are contiguous, it will directly convert both to 1D array and parallelize the operation
///
/// - otherwise, it will use the strided map to parallelize the operation
#[cfg_attr(feature = "track_caller", track_caller)]
pub fn binary_fn<A, B, K, F>(lhs: &_Tensor<A>, rhs: &_Tensor<B>, f: F) -> anyhow::Result<_Tensor<K>>
where
    A: CommonBounds,
    B: CommonBounds,
    K: CommonBounds,
    F: Fn(A, B) -> K + Sync + Send + Copy,
{
    if lhs.size() == 1 {
        let val = lhs.as_raw()[0];
        let res = _Tensor::<K, Cpu>::empty(rhs.shape())?;
        if rhs.parent().is_some() {
            res.par_iter_mut().zip(rhs.par_iter()).for_each(|(a, b)| {
                *a = f(val, b);
            });
        } else {
            res.as_raw_mut()
                .par_iter_mut()
                .zip(rhs.as_raw().par_iter())
                .for_each(|(a, &b)| {
                    *a = f(val, b);
                });
        }
        Ok(res)
    } else if rhs.size() == 1 {
        let val = rhs.as_raw()[0];
        let res = _Tensor::<K, Cpu>::empty(lhs.shape())?;
        if lhs.parent().is_some() {
            res.par_iter_mut().zip(lhs.par_iter()).for_each(|(a, b)| {
                *a = f(b, val);
            });
        } else {
            res.as_raw_mut()
                .par_iter_mut()
                .zip(lhs.as_raw().par_iter())
                .for_each(|(a, &b)| {
                    *a = f(b, val);
                });
        }
        Ok(res)
    } else {
        if rhs.parent().is_none()
            && lhs.parent().is_none()
            && rhs.is_contiguous()
            && lhs.is_contiguous()
            && rhs.shape() == lhs.shape()
        {
            let res_shape = predict_broadcast_shape(lhs.shape(), rhs.shape())?;
            let ret;
            ret = _Tensor::<K, Cpu>::empty(res_shape)?;
            ret.as_raw_mut()
                .par_iter_mut()
                .zip(lhs.as_raw().par_iter())
                .zip(rhs.as_raw().par_iter())
                .for_each(|((ret, &lhs), &rhs)| {
                    *ret = f(lhs, rhs);
                });
            Ok(ret)
        } else {
            let ret = lhs
                .par_iter()
                .zip(rhs.par_iter())
                .strided_map(|(x, y)| f(x, y))
                .collect::<_Tensor<K>>();
            Ok(ret)
        }
    }
}

/// same function as `binary_fn`, just the output tensor is passed as an argument
///
/// full documentation can be found in `binary_fn`
pub fn binary_fn_with_out<A, B, O, Q, K, F>(
    lhs: &_Tensor<A>,
    rhs: &_Tensor<B>,
    f: F,
    out: O,
) -> anyhow::Result<_Tensor<K>>
where
    A: CommonBounds,
    B: CommonBounds,
    O: Borrow<_Tensor<Q>>,
    K: CommonBounds,
    Q: CommonBounds,
    F: Fn(A, B) -> K + Sync + Send + Copy,
{
    if lhs.size() == 1 {
        let val = lhs.as_raw()[0];
        let ret = if out.borrow().size() * size_of::<Q>() != rhs.size() * size_of::<B>() {
            _Tensor::<K, Cpu>::empty(rhs.shape())?
        } else {
            out.borrow().static_cast::<K>()?
        };
        ret.as_raw_mut()
            .par_iter_mut()
            .zip(rhs.as_raw().par_iter())
            .for_each(|(a, &b)| {
                *a = f(val, b);
            });
        Ok(ret)
    } else if rhs.size() == 1 {
        let val = rhs.as_raw()[0];
        let ret;
        if out.borrow().size() * size_of::<Q>() != lhs.size() * size_of::<A>() {
            ret = _Tensor::<K, Cpu>::empty(lhs.shape())?;
        } else {
            ret = _Tensor::<K, Cpu>::empty(lhs.shape())?;
        }
        ret.as_raw_mut()
            .par_iter_mut()
            .zip(lhs.as_raw().par_iter())
            .for_each(|(a, &b)| {
                *a = f(b, val);
            });
        Ok(ret)
    } else {
        let res_shape = predict_broadcast_shape(lhs.shape(), rhs.shape())?;
        let ret;
        let ret_size: usize = res_shape.iter().product::<i64>() as usize;
        if out.borrow().size() * size_of::<Q>() != ret_size * size_of::<A>() {
            ret = _Tensor::<K, Cpu>::empty(res_shape)?;
        } else {
            ret = _Tensor::<K, Cpu>::empty(res_shape)?;
        }
        if rhs.is_contiguous() && lhs.is_contiguous() && rhs.shape() == lhs.shape() {
            let min_len: usize =
                ret.size() / (((rayon::current_num_threads() as f64) * 1.3) as usize);
            ret.as_raw_mut()
                .par_iter_mut()
                .with_min_len(min_len)
                .zip(lhs.as_raw().par_iter().with_min_len(min_len))
                .zip(rhs.as_raw().par_iter().with_min_len(min_len))
                .for_each(|((ret, &lhs), &rhs)| {
                    *ret = f(lhs, rhs);
                });
        } else {
            ret.par_iter_mut()
                .zip(lhs.par_iter().zip(rhs.par_iter()))
                .for_each(|(res, (x, y))| {
                    *res = f(x, y);
                });
        }
        Ok(ret)
    }
}

use std::borrow::Borrow;
/// same function as `binary_fn_simd`, just the output tensor is passed as an argument
///
/// full documentation can be found in `binary_fn_simd`
///
/// simd will be enabled only when all operands and output type have the same vector size.
#[cfg(feature = "simd")]
#[cfg_attr(feature = "track_caller", track_caller)]
pub fn binary_fn_with_out_simd<A, B, O, Q, K, F, F2>(
    lhs: &_Tensor<A>,
    rhs: &_Tensor<B>,
    f: F,
    f2: F2,
    out: O,
) -> anyhow::Result<_Tensor<K>>
where
    A: CommonBounds,
    B: CommonBounds,
    O: TensorLike<Q> + TensorInfo<Q> + Borrow<_Tensor<Q>>,
    K: CommonBounds,
    Q: CommonBounds,
    F: Fn(A, B) -> K + Sync + Send + Copy,
    F2: Fn(<A as TypeCommon>::Vec, <B as TypeCommon>::Vec) -> <K as TypeCommon>::Vec
        + Sync
        + Send
        + Copy,
{

    use rayon::slice::{ParallelSlice, ParallelSliceMut};
    use tensor_types::traits::*;
    if lhs.size() == 1 {
        let val = lhs.as_raw()[0];
        let val_vec = <A as TypeCommon>::Vec::splat(val);
        let res = if out.size() * size_of::<Q>() != rhs.size() * size_of::<B>() {
            _Tensor::<K, Cpu>::empty(rhs.shape())?
        } else {
            out.borrow().static_cast::<K>()?
        };
        if <A as TypeCommon>::Vec::SIZE == <B as TypeCommon>::Vec::SIZE
            && <B as TypeCommon>::Vec::SIZE == <K as TypeCommon>::Vec::SIZE
        {
            let remain = res.size() % <A as TypeCommon>::Vec::SIZE;
            res.as_raw_mut()
                .par_chunks_exact_mut(<A as TypeCommon>::Vec::SIZE)
                .zip(rhs.as_raw().par_chunks_exact(<A as TypeCommon>::Vec::SIZE))
                .for_each(|(a, b)| {
                    let inp = unsafe { <B as TypeCommon>::Vec::from_ptr(b.as_ptr()) };
                    let res: *const K = f2(val_vec, inp).as_ptr();
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            res,
                            a.as_mut_ptr(),
                            <A as TypeCommon>::Vec::SIZE,
                        );
                    }
                });
            if remain > 0 {
                res.as_raw_mut()[res.size() - remain..]
                    .iter_mut()
                    .zip(rhs.as_raw()[res.size() - remain..].iter())
                    .for_each(|(a, b)| {
                        *a = f(val, *b);
                    });
            }
        } else {
            res.as_raw_mut()
                .par_chunks_exact_mut(<K as TypeCommon>::Vec::SIZE)
                .zip(rhs.as_raw().par_chunks_exact(<K as TypeCommon>::Vec::SIZE))
                .for_each(|(a, b)| {
                    a.iter_mut().zip(b.iter()).for_each(|(a, b)| {
                        *a = f(val, *b);
                    });
                });
            let remain = res.size() % <K as TypeCommon>::Vec::SIZE;
            if remain > 0 {
                res.as_raw_mut()[res.size() - remain..]
                    .iter_mut()
                    .zip(rhs.as_raw()[res.size() - remain..].iter())
                    .for_each(|(a, b)| {
                        *a = f(val, *b);
                    });
            }
        }
        Ok(res)
    } else if rhs.size() == 1 {
        let val = rhs.as_raw()[0];
        let val_vec = <B as TypeCommon>::Vec::splat(val);
        let res = if out.size() * size_of::<Q>() != lhs.size() * size_of::<B>() {
            _Tensor::<K, Cpu>::empty(lhs.shape())?
        } else {
            _Tensor::<K, Cpu>::empty(lhs.shape())?
        };
        if <A as TypeCommon>::Vec::SIZE == <B as TypeCommon>::Vec::SIZE
            && <B as TypeCommon>::Vec::SIZE == <K as TypeCommon>::Vec::SIZE
        {
            let remain = res.size() % <A as TypeCommon>::Vec::SIZE;
            res.as_raw_mut()
                .par_chunks_exact_mut(<A as TypeCommon>::Vec::SIZE)
                .zip(lhs.as_raw().par_chunks_exact(<A as TypeCommon>::Vec::SIZE))
                .for_each(|(a, lhs)| {
                    let inp = unsafe { <A as TypeCommon>::Vec::from_ptr(lhs.as_ptr()) };
                    let res: *const K = f2(inp, val_vec).as_ptr();
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            res,
                            a.as_mut_ptr(),
                            <A as TypeCommon>::Vec::SIZE,
                        );
                    }
                });
            if remain > 0 {
                res.as_raw_mut()[res.size() - remain..]
                    .iter_mut()
                    .zip(lhs.as_raw()[res.size() - remain..].iter())
                    .for_each(|(a, lhs)| {
                        *a = f(*lhs, val);
                    });
            }
        } else {
            res.as_raw_mut()
                .par_chunks_exact_mut(<K as TypeCommon>::Vec::SIZE)
                .zip(lhs.as_raw().par_chunks_exact(<K as TypeCommon>::Vec::SIZE))
                .for_each(|(a, lhs)| {
                    a.iter_mut().zip(lhs.iter()).for_each(|(a, lhs)| {
                        *a = f(*lhs, val);
                    });
                });
            let remain = res.size() % <K as TypeCommon>::Vec::SIZE;
            if remain > 0 {
                res.as_raw_mut()[res.size() - remain..]
                    .iter_mut()
                    .zip(lhs.as_raw()[res.size() - remain..].iter())
                    .for_each(|(a, lhs)| {
                        *a = f(*lhs, val);
                    });
            }
        }
        Ok(res)
    } else {
        if rhs.is_contiguous() && lhs.is_contiguous() && rhs.shape() == lhs.shape() {
            let ret = if out.size() * size_of::<Q>() != rhs.size() * size_of::<B>() {
                _Tensor::<K, Cpu>::empty(rhs.shape())?
            } else {
                _Tensor::<K, Cpu>::empty(rhs.shape())?
            };
            if <A as TypeCommon>::Vec::SIZE == <B as TypeCommon>::Vec::SIZE
                && <B as TypeCommon>::Vec::SIZE == <K as TypeCommon>::Vec::SIZE
            {
                let remain = ret.size() % <K as TypeCommon>::Vec::SIZE;
                ret.as_raw_mut()
                    .par_chunks_exact_mut(<K as TypeCommon>::Vec::SIZE)
                    .zip(lhs.as_raw().par_chunks_exact(<K as TypeCommon>::Vec::SIZE))
                    .zip(rhs.as_raw().par_chunks_exact(<K as TypeCommon>::Vec::SIZE))
                    .for_each(|((ret, lhs), rhs)| {
                        let a = unsafe { <A as TypeCommon>::Vec::from_ptr(lhs.as_ptr()) };
                        let b = unsafe { <B as TypeCommon>::Vec::from_ptr(rhs.as_ptr()) };
                        let res = f2(a, b);
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                res.as_ptr(),
                                ret.as_mut_ptr(),
                                <K as TypeCommon>::Vec::SIZE,
                            );
                        }
                    });
                if remain > 0 {
                    ret.as_raw_mut()[ret.size() - remain..]
                        .iter_mut()
                        .zip(lhs.as_raw()[ret.size() - remain..].iter())
                        .zip(rhs.as_raw()[ret.size() - remain..].iter())
                        .for_each(|((a, &lhs), &rhs)| {
                            *a = f(lhs, rhs);
                        });
                }
            } else {
                let min_len: usize =
                    ret.size() / (((rayon::current_num_threads() as f64) * 1.3) as usize);
                ret.as_raw_mut()
                    .par_iter_mut()
                    .with_min_len(min_len)
                    .zip(lhs.as_raw().par_iter().with_min_len(min_len))
                    .zip(rhs.as_raw().par_iter().with_min_len(min_len))
                    .for_each(|((ret, &lhs), &rhs)| {
                        *ret = f(lhs, rhs);
                    });
            }
            Ok(ret)
        } else {
            if <A as TypeCommon>::Vec::SIZE == <B as TypeCommon>::Vec::SIZE
                && <B as TypeCommon>::Vec::SIZE == <K as TypeCommon>::Vec::SIZE
            {
                let ret = lhs
                    .par_iter_simd()
                    .zip(rhs.par_iter_simd())
                    .strided_map(
                        |(res, (x, y))| {
                            *res = f(x, y);
                        },
                        |(res, (x, y))| {
                            let x_ptr = x.as_ptr();
                            let y_ptr = y.as_ptr();
                            *res = f2(unsafe { <A as TypeCommon>::Vec::from_ptr(x_ptr) }, unsafe {
                                <B as TypeCommon>::Vec::from_ptr(y_ptr)
                            });
                        },
                    )
                    .collect::<_Tensor<K>>();
                Ok(ret)
            } else {
                let ret = lhs
                    .par_iter()
                    .zip(rhs.par_iter())
                    .strided_map(|(x, y)| f(x, y))
                    .collect::<_Tensor<K>>();
                Ok(ret)
            }
        }
    }
}

/// simd version of `binary_fn`
///
/// simd will be enabled only when all operands and output type have the same vector size.
///
/// full documentation can be found in `binary_fn`
#[cfg(feature = "simd")]
#[cfg_attr(feature = "track_caller", track_caller)]
pub fn binary_fn_simd<A, B, K, F, F2>(
    lhs: &_Tensor<A>,
    rhs: &_Tensor<B>,
    f: F,
    f2: F2,
) -> anyhow::Result<_Tensor<K>>
where
    A: CommonBounds,
    B: CommonBounds,
    K: CommonBounds,
    F: Fn(A, B) -> K + Sync + Send + Copy,
    F2: Fn(<A as TypeCommon>::Vec, <B as TypeCommon>::Vec) -> <K as TypeCommon>::Vec
        + Sync
        + Send
        + Copy,
{
    use rayon::slice::{ParallelSlice, ParallelSliceMut};
    use tensor_types::vectors::traits::*;

    // for binary, case could be (scalar op tensor) or (tensor op scalar) or (tensor op tensor)
    // scalr can be store in to the register to achieve better performance
    if lhs.size() == 1 {
        let val = lhs.as_raw()[0];
        let val_vec = <A as TypeCommon>::Vec::splat(val);
        let res = _Tensor::<K, Cpu>::empty(rhs.shape())?;
        if rhs.parent().is_some() {
            res.par_iter_mut().zip(rhs.par_iter()).for_each(|(a, b)| {
                *a = f(val, b);
            });
            return Ok(res);
        }
        // simd is enabled only when all operands and output type have the same vector size.
        // example: f32x4 + f32x4 = f32x4 or f32x8 + i32x8 = f32x8. f32x4 + f32x8 is not allowed.
        if <A as TypeCommon>::Vec::SIZE == <B as TypeCommon>::Vec::SIZE
            && <B as TypeCommon>::Vec::SIZE == <K as TypeCommon>::Vec::SIZE
        {
            let remain = res.size() % <A as TypeCommon>::Vec::SIZE;
            res.as_raw_mut()
                .par_chunks_exact_mut(<A as TypeCommon>::Vec::SIZE)
                .zip(rhs.as_raw().par_chunks_exact(<A as TypeCommon>::Vec::SIZE))
                .for_each(|(a, b)| {
                    let inp = unsafe { <B as TypeCommon>::Vec::from_ptr(b.as_ptr()) };
                    let res: *const K = f2(val_vec, inp).as_ptr();
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            res,
                            a.as_mut_ptr(),
                            <A as TypeCommon>::Vec::SIZE,
                        );
                    }
                });
            if remain > 0 {
                res.as_raw_mut()[res.size() - remain..]
                    .iter_mut()
                    .zip(rhs.as_raw()[res.size() - remain..].iter())
                    .for_each(|(a, b)| {
                        *a = f(val, *b);
                    });
            }
        } else {
            res.as_raw_mut()
                .par_chunks_exact_mut(<K as TypeCommon>::Vec::SIZE)
                .zip(rhs.as_raw().par_chunks_exact(<K as TypeCommon>::Vec::SIZE))
                .for_each(|(a, b)| {
                    a.iter_mut().zip(b.iter()).for_each(|(a, b)| {
                        *a = f(val, *b);
                    });
                });
            let remain = res.size() % <K as TypeCommon>::Vec::SIZE;
            if remain > 0 {
                res.as_raw_mut()[res.size() - remain..]
                    .iter_mut()
                    .zip(rhs.as_raw()[res.size() - remain..].iter())
                    .for_each(|(a, b)| {
                        *a = f(val, *b);
                    });
            }
        }
        Ok(res)
    } else if rhs.size() == 1 {
        let val = rhs.as_raw()[0];
        let val_vec = <B as TypeCommon>::Vec::splat(val);
        let res = _Tensor::<K, Cpu>::empty(lhs.shape())?;
        if lhs.parent().is_some() {
            res.par_iter_mut().zip(lhs.par_iter()).for_each(|(a, b)| {
                *a = f(b, val);
            });
            return Ok(res);
        }
        // simd is enabled only when all operands and output type have the same vector size.
        // example: f32x4 + f32x4 = f32x4 or f32x8 + i32x8 = f32x8. f32x4 + f32x8 is not allowed.
        if <A as TypeCommon>::Vec::SIZE == <B as TypeCommon>::Vec::SIZE
            && <B as TypeCommon>::Vec::SIZE == <K as TypeCommon>::Vec::SIZE
        {
            let remain = res.size() % <A as TypeCommon>::Vec::SIZE;
            res.as_raw_mut()
                .par_chunks_exact_mut(<A as TypeCommon>::Vec::SIZE)
                .zip(lhs.as_raw().par_chunks_exact(<A as TypeCommon>::Vec::SIZE))
                .for_each(|(a, lhs)| {
                    let inp = unsafe { <A as TypeCommon>::Vec::from_ptr(lhs.as_ptr()) };
                    let res: *const K = f2(inp, val_vec).as_ptr();
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            res,
                            a.as_mut_ptr(),
                            <A as TypeCommon>::Vec::SIZE,
                        );
                    }
                });
            if remain > 0 {
                res.as_raw_mut()[res.size() - remain..]
                    .iter_mut()
                    .zip(lhs.as_raw()[res.size() - remain..].iter())
                    .for_each(|(a, lhs)| {
                        *a = f(*lhs, val);
                    });
            }
        } else {
            res.as_raw_mut()
                .par_chunks_exact_mut(<K as TypeCommon>::Vec::SIZE)
                .zip(lhs.as_raw().par_chunks_exact(<K as TypeCommon>::Vec::SIZE))
                .for_each(|(a, lhs)| {
                    a.iter_mut().zip(lhs.iter()).for_each(|(a, lhs)| {
                        *a = f(*lhs, val);
                    });
                });
            let remain = res.size() % <K as TypeCommon>::Vec::SIZE;
            if remain > 0 {
                res.as_raw_mut()[res.size() - remain..]
                    .iter_mut()
                    .zip(lhs.as_raw()[res.size() - remain..].iter())
                    .for_each(|(a, lhs)| {
                        *a = f(*lhs, val);
                    });
            }
        }
        Ok(res)
    } else {
        // if both lhs and rhs are contiguous and have the same shape, we can directly convert both to 1D array and parrallelize the operation
        if rhs.is_contiguous()
            && lhs.is_contiguous()
            && rhs.parent().is_none()
            && lhs.parent().is_none()
            && rhs.shape() == lhs.shape()
        {
            let ret;
            ret = _Tensor::<K, Cpu>::empty(rhs.shape())?;
            if <A as TypeCommon>::Vec::SIZE == <B as TypeCommon>::Vec::SIZE
                && <B as TypeCommon>::Vec::SIZE == <K as TypeCommon>::Vec::SIZE
            {
                // chunk the array, the chunk size must be a multiple of the vector size
                let per_thread_len = ret.size() / rayon::current_num_threads();
                let per_thread_remain = per_thread_len % <K as TypeCommon>::Vec::SIZE;
                let per_thread_real_len = per_thread_len - per_thread_remain;
                let remain = ret.size() % per_thread_real_len;
                ret.as_raw_mut()
                    .par_chunks_exact_mut(per_thread_real_len)
                    .zip(lhs.as_raw().par_chunks_exact(per_thread_real_len))
                    .zip(rhs.as_raw().par_chunks_exact(per_thread_real_len))
                    .for_each(|((ret, lhs), rhs)| {
                        ret.chunks_exact_mut(<A as TypeCommon>::Vec::SIZE)
                            .zip(lhs.chunks_exact(<A as TypeCommon>::Vec::SIZE))
                            .zip(rhs.chunks_exact(<A as TypeCommon>::Vec::SIZE))
                            .for_each(|((ret, lhs), rhs)| {
                                let a = unsafe { <A as TypeCommon>::Vec::from_ptr(lhs.as_ptr()) };
                                let b = unsafe { <B as TypeCommon>::Vec::from_ptr(rhs.as_ptr()) };
                                let res = f2(a, b);
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        res.as_ptr(),
                                        ret.as_mut_ptr(),
                                        <K as TypeCommon>::Vec::SIZE,
                                    );
                                }
                            });
                    });
                // handle the remaining elements
                if remain > 0 {
                    ret.as_raw_mut()[ret.size() - remain..]
                        .iter_mut()
                        .zip(lhs.as_raw()[ret.size() - remain..].iter())
                        .zip(rhs.as_raw()[ret.size() - remain..].iter())
                        .for_each(|((a, &lhs), &rhs)| {
                            *a = f(lhs, rhs);
                        });
                }
            } else {
                let min_len: usize =
                    ret.size() / (((rayon::current_num_threads() as f64) * 1.3) as usize);
                ret.as_raw_mut()
                    .par_iter_mut()
                    .with_min_len(min_len)
                    .zip(lhs.as_raw().par_iter().with_min_len(min_len))
                    .zip(rhs.as_raw().par_iter().with_min_len(min_len))
                    .for_each(|((ret, &lhs), &rhs)| {
                        *ret = f(lhs, rhs);
                    });
            }
            Ok(ret)
        } else {
            // simd is enabled only when all operands and output type have the same vector size.
            // example: f32x4 + f32x4 = f32x4 or f32x8 + i32x8 = f32x8. f32x4 + f32x8 is not allowed.
            if <A as TypeCommon>::Vec::SIZE == <B as TypeCommon>::Vec::SIZE
                && <B as TypeCommon>::Vec::SIZE == <K as TypeCommon>::Vec::SIZE
            {
                let ret = lhs
                    .par_iter_simd()
                    .zip(rhs.par_iter_simd())
                    .strided_map(
                        |(res, (x, y))| {
                            *res = f(x, y);
                        },
                        |(res, (x, y))| {
                            *res = f2(x, y);
                        },
                    )
                    .collect::<_Tensor<K>>();
                Ok(ret)
            } else {
                // because the computation can't be parallelized and opeands are not contiguous, we have to use parallel strided
                let ret = lhs
                    .par_iter()
                    .zip(rhs.par_iter())
                    .strided_map(|(x, y)| f(x, y))
                    .collect::<_Tensor<K>>();
                Ok(ret)
            }
        }
    }
}
