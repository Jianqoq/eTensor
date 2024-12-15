use crate::ops::cuda::cuda_utils::compile_kernel;
use crate::ops::cuda::cuda_utils::compute_kernel_launch_config;
use crate::tensor_base::_Tensor;
use crate::Cuda;
use cudarc::driver::DeviceRepr;
use cudarc::driver::LaunchAsync;
use std::borrow::Borrow;
use std::panic::Location;
use tensor_common::err_handler::ErrHandler::InvalidOutSize;
use tensor_common::shape::Shape;
use tensor_traits::tensor::CommonBounds;
use tensor_traits::tensor::TensorCreator;
use tensor_traits::tensor::TensorInfo;
use tensor_traits::TensorLike;
use tensor_types::dtype::TypeCommon;

/// Performs a binary operation on two tensors with optional SIMD optimization and an output tensor.
///
/// This method applies a binary function element-wise on two tensors (`lhs` and `rhs`) and returns
/// a new tensor with the result. Optionally, SIMD (Single Instruction, Multiple Data) can be used
/// for vectorized operations if the sizes of the underlying data vectors align. Additionally,
/// the user can provide an output tensor to store the result, allowing in-place computations
/// and reducing memory allocations.
///
/// # Arguments
///
/// * `lhs` - A reference to the left-hand side tensor involved in the binary operation.
/// * `rhs` - A reference to the right-hand side tensor involved in the binary operation.
/// * `f` - A binary function applied to elements of the tensors during the operation. This function
///   is used when SIMD is not applicable.
/// * `f2` - A binary function that operates on vectorized data (SIMD). This function is used when
///   SIMD is applicable.
/// * `out` - An optional output tensor that, if provided, will store the result of the operation.
///   If not provided, a new tensor will be created to hold the result.
///
/// # Returns
///
/// Returns a `Result` containing a new tensor with the result of the binary operation. If any error occurs
/// (e.g., shape mismatch or allocation issues), an `anyhow::Result` with an error message is returned.
///
/// # SIMD Optimization
///
/// If the vector sizes of the input tensors match and SIMD is enabled, the `f2` function is applied to
/// perform vectorized operations for faster computation. If not, the scalar function `f` is applied to each element.
#[cfg_attr(feature = "track_caller", track_caller)]
pub(crate) fn binary_fn_with_out_simd<A, B, O, K, F, const CUDA_DEVICE: usize>(
    lhs: &_Tensor<A, Cuda, CUDA_DEVICE>,
    rhs: &_Tensor<B, Cuda, CUDA_DEVICE>,
    f: F,
    out: Option<O>,
) -> anyhow::Result<_Tensor<K, Cuda, CUDA_DEVICE>>
where
    A: CommonBounds + DeviceRepr,
    B: CommonBounds + DeviceRepr,
    O: Borrow<_Tensor<K, Cuda, CUDA_DEVICE>>,
    K: CommonBounds + DeviceRepr,
    F: Fn(&String, &String) -> String,
{
    let module_name = format!(
        "bn{}{}{}{}{}{}",
        A::ID,
        lhs.layout.shape(),
        lhs.layout.strides(),
        B::ID,
        rhs.layout.shape(),
        rhs.layout.strides()
    );
    if lhs.size() == 1 {
        let val = lhs.to_cpu()?.as_raw()[0];
        let res = extract_out::<B, K, O, CUDA_DEVICE>(rhs.size(), rhs.shape(), out)?;
        if rhs.is_contiguous() {
            let map = compile_kernel(
                &module_name,
                &format!(
                    "extern \"C\" __global__ void lhs_scalar_rhs_contiguous({} *out, {} lhs, {} *rhs)
                    {{
                        size_t idx = blockIdx.x * blockDim.x + threadIdx.x;
                        size_t stride = blockDim.x * gridDim.x;
                        while (idx < {})
                        {{
                            out[idx] = {};
                            idx += stride;
                        }}
                    }}",
                    K::CUDA_TYPE,
                    A::CUDA_TYPE,
                    B::CUDA_TYPE,
                    res.size(),
                    f(&format!("({})lhs", K::CUDA_TYPE), &format!("({})rhs[idx]", K::CUDA_TYPE))
                ),
                res.device(),
                &["lhs_scalar_rhs_contiguous"],
            )?;
            let kernel = res
                .device()
                .get_func(&module_name, "lhs_scalar_rhs_contiguous")
                .unwrap();
            let mut out_slice = unsafe {
                res.device()
                    .upgrade_device_ptr::<K>(res.ptr().ptr as u64, res.size())
            };
            let rhs_slice = unsafe {
                rhs.device()
                    .upgrade_device_ptr::<B>(rhs.ptr().ptr as u64, rhs.size())
            };
            let reg_info = map
                .get("lhs_scalar_rhs_contiguous")
                .expect("func_name not found");
            let cfg = compute_kernel_launch_config(res.device(), reg_info, res.size());
            unsafe { kernel.launch(cfg, (&mut out_slice, val, &rhs_slice)) }?;
            out_slice.leak();
            rhs_slice.leak();
        } else {
            let rhs_broadcast_layout = rhs.layout.to_broadcast_layout(res.shape())?;
            let shape_str = rhs_broadcast_layout
                .shape()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let strides_str = rhs_broadcast_layout
                .strides()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let map = compile_kernel(
                &module_name,
                &format!(
                    "extern \"C\" __global__ void lhs_scalar_rhs_contiguous({} *out, {} lhs, {} *rhs)
                    {{
                        size_t idx = blockIdx.x * blockDim.x + threadIdx.x;
                        size_t stride = blockDim.x * gridDim.x;
                        __constant__ long long shape[] = {{{}}};
                        __constant__ long long strides[] = {{{}}};
                        while (idx < {})
                        {{
                            long amount = idx;
                            long offset = 0;
                            #pragma unroll
                            for (int j = {} - 1; j >= 0; j--)
                            {{
                                offset += amount % shape[j] * strides[j];
                                amount /= shape[j];
                            }}
                            out[idx] = {};
                            idx += stride;
                        }}
                    }}",
                    K::CUDA_TYPE,
                    A::CUDA_TYPE,
                    B::CUDA_TYPE,
                    shape_str,
                    strides_str,
                    res.size(),
                    res.ndim(),
                    f(&format!("({})lhs", K::CUDA_TYPE), &format!("({})rhs[offset]", K::CUDA_TYPE))
                ),
                res.device(),
                &["lhs_scalar_rhs_not_contiguous"],
            )?;
            let kernel = res
                .device()
                .get_func(&module_name, "lhs_scalar_rhs_not_contiguous")
                .unwrap();
            let mut out_slice = unsafe {
                res.device()
                    .upgrade_device_ptr::<K>(res.ptr().ptr as u64, res.size())
            };
            let rhs_slice = unsafe {
                rhs.device()
                    .upgrade_device_ptr::<B>(rhs.ptr().ptr as u64, rhs.size())
            };
            let reg_info = map
                .get("lhs_scalar_rhs_not_contiguous")
                .expect("func_name not found");
            let cfg = compute_kernel_launch_config(res.device(), reg_info, res.size());
            unsafe { kernel.launch(cfg, (&mut out_slice, val, &rhs_slice)) }?;
            out_slice.leak();
            rhs_slice.leak();
        }
        Ok(res)
    } else if rhs.size() == 1 {
        let val = rhs.to_cpu()?.as_raw()[0];
        let res = extract_out::<A, K, O, CUDA_DEVICE>(lhs.size(), lhs.shape(), out)?;
        if lhs.is_contiguous() {
            let map = compile_kernel(
                &module_name,
                &format!(
                    "extern \"C\" __global__ void rhs_scalar_lhs_contiguous({} *out, {} *lhs, {} rhs)
                    {{
                        size_t idx = blockIdx.x * blockDim.x + threadIdx.x;
                        size_t stride = blockDim.x * gridDim.x;
                        while (idx < {})
                        {{
                            out[idx] = {};
                            idx += stride;
                        }}
                    }}",
                    K::CUDA_TYPE,
                    A::CUDA_TYPE,
                    B::CUDA_TYPE,
                    res.size(),
                    f(&format!("({})lhs[idx]", K::CUDA_TYPE), &format!("({})rhs", K::CUDA_TYPE))
                ),
                res.device(),
                &["rhs_scalar_lhs_contiguous"],
            )?;
            let kernel = res
                .device()
                .get_func(&module_name, "rhs_scalar_lhs_contiguous")
                .unwrap();
            let mut out_slice = unsafe {
                res.device()
                    .upgrade_device_ptr::<K>(res.ptr().ptr as u64, res.size())
            };
            let lhs_slice = unsafe {
                lhs.device()
                    .upgrade_device_ptr::<A>(lhs.ptr().ptr as u64, lhs.size())
            };
            let reg_info = map
                .get("rhs_scalar_lhs_contiguous")
                .expect("func_name not found");
            let cfg = compute_kernel_launch_config(res.device(), reg_info, res.size());
            unsafe { kernel.launch(cfg, (&mut out_slice, &lhs_slice, val)) }?;
            out_slice.leak();
            lhs_slice.leak();
        } else {
            let lhs_broadcast_layout = lhs.layout.to_broadcast_layout(res.shape())?;
            let shape_str = lhs_broadcast_layout
                .shape()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let strides_str = lhs_broadcast_layout
                .strides()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let map = compile_kernel(
                &module_name,
                &format!(
                    "extern \"C\" __global__ void rhs_scalar_lhs_contiguous({} *out, {} *lhs, {} rhs)
                    {{
                        size_t idx = blockIdx.x * blockDim.x + threadIdx.x;
                        size_t stride = blockDim.x * gridDim.x;
                        __constant__ long long shape[] = {{{}}};
                        __constant__ long long strides[] = {{{}}};
                        while (idx < {})
                        {{
                            long amount = idx;
                            long offset = 0;
                            #pragma unroll
                            for (int j = {} - 1; j >= 0; j--)
                            {{
                                offset += amount % shape[j] * strides[j];
                                amount /= shape[j];
                            }}
                            out[idx] = {};
                            idx += stride;
                        }}
                    }}",
                    K::CUDA_TYPE,
                    A::CUDA_TYPE,
                    B::CUDA_TYPE,
                    shape_str,
                    strides_str,
                    res.size(),
                    res.ndim(),
                    f(&format!("({})lhs[offset]", K::CUDA_TYPE), &format!("({})rhs", K::CUDA_TYPE))
                ),
                res.device(),
                &["rhs_scalar_lhs_not_contiguous"],
            )?;
            let kernel = res
                .device()
                .get_func(&module_name, "rhs_scalar_lhs_not_contiguous")
                .unwrap();
            let mut out_slice = unsafe {
                res.device()
                    .upgrade_device_ptr::<K>(res.ptr().ptr as u64, res.size())
            };
            let lhs_slice = unsafe {
                lhs.device()
                    .upgrade_device_ptr::<A>(lhs.ptr().ptr as u64, lhs.size())
            };
            let reg_info = map
                .get("rhs_scalar_lhs_not_contiguous")
                .expect("func_name not found");
            let cfg = compute_kernel_launch_config(res.device(), reg_info, res.size());
            unsafe { kernel.launch(cfg, (&mut out_slice, &lhs_slice, val)) }?;
            out_slice.leak();
            lhs_slice.leak();
        }
        Ok(res)
    } else {
        if rhs.is_contiguous() && lhs.is_contiguous() && rhs.shape() == lhs.shape() {
            let res = extract_out::<B, K, O, CUDA_DEVICE>(rhs.size(), rhs.shape(), out)?;
            let map = compile_kernel(
                &module_name,
                &format!(
                    "extern \"C\" __global__ void lhs_scalar_rhs_contiguous({} *out, {} *lhs, {} *rhs)
                    {{
                        size_t idx = blockIdx.x * blockDim.x + threadIdx.x;
                        size_t stride = blockDim.x * gridDim.x;
                        while (idx < {})
                        {{
                            out[idx] = {};
                            idx += stride;
                        }}
                    }}",
                    K::CUDA_TYPE,
                    A::CUDA_TYPE,
                    B::CUDA_TYPE,
                    res.size(),
                    f(&format!("({})lhs[idx]", K::CUDA_TYPE), &format!("({})rhs[idx]", K::CUDA_TYPE))
                ),
                res.device(),
                &["lhs_scalar_rhs_contiguous"],
            )?;
            let kernel = res
                .device()
                .get_func(&module_name, "lhs_scalar_rhs_contiguous")
                .unwrap();
            let mut out_slice = unsafe {
                res.device()
                    .upgrade_device_ptr::<K>(res.ptr().ptr as u64, res.size())
            };
            let rhs_slice = unsafe {
                rhs.device()
                    .upgrade_device_ptr::<B>(rhs.ptr().ptr as u64, rhs.size())
            };
            let lhs_slice = unsafe {
                lhs.device()
                    .upgrade_device_ptr::<A>(lhs.ptr().ptr as u64, lhs.size())
            };
            let reg_info = map
                .get("lhs_scalar_rhs_contiguous")
                .expect("func_name not found");
            let cfg = compute_kernel_launch_config(res.device(), reg_info, res.size());
            unsafe { kernel.launch(cfg, (&mut out_slice, &lhs_slice, &rhs_slice)) }?;
            out_slice.leak();
            lhs_slice.leak();
            rhs_slice.leak();
            Ok(res)
        } else {
            let res_layout = lhs.layout.broadcast(&rhs.layout)?;
            let lhs_broadcast_layout = lhs.layout.to_broadcast_layout(res_layout.shape())?;
            let rhs_broadcast_layout = rhs.layout.to_broadcast_layout(res_layout.shape())?;
            let res = extract_out::<K, K, O, CUDA_DEVICE>(
                res_layout.size() as usize,
                res_layout.shape(),
                out,
            )?;
            let lhs_shape_str = lhs_broadcast_layout
                .shape()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let lhs_strides_str = lhs_broadcast_layout
                .strides()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let rhs_shape_str = rhs_broadcast_layout
                .shape()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let rhs_strides_str = rhs_broadcast_layout
                .strides()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let map = compile_kernel(
                &module_name,
                &format!(
                    "extern \"C\" __global__ void binop({} *out, {} *lhs, {} *rhs)
                    {{
                        size_t idx = blockIdx.x * blockDim.x + threadIdx.x;
                        size_t stride = blockDim.x * gridDim.x;
                        __constant__ long long lhs_shape[] = {{{}}};
                        __constant__ long long lhs_strides[] = {{{}}};
                        __constant__ long long rhs_shape[] = {{{}}};
                        __constant__ long long rhs_strides[] = {{{}}};
                        while (idx < {})
                        {{
                            long lhs_amount = idx;
                            long lhs_offset = 0;
                            long rhs_amount = idx;
                            long rhs_offset = 0;
                            #pragma unroll
                            for (int j = {} - 1; j >= 0; j--)
                            {{
                                lhs_offset += lhs_amount % lhs_shape[j] * lhs_strides[j];
                                lhs_amount /= lhs_shape[j];
                                rhs_offset += rhs_amount % rhs_shape[j] * rhs_strides[j];
                                rhs_amount /= rhs_shape[j];
                            }}
                            out[idx] = {};
                            idx += stride;
                        }}
                    }}",
                    K::CUDA_TYPE,
                    A::CUDA_TYPE,
                    B::CUDA_TYPE,
                    lhs_shape_str,
                    lhs_strides_str,
                    rhs_shape_str,
                    rhs_strides_str,
                    res.size(),
                    res.ndim(),
                    f(&format!("({})lhs[lhs_offset]", K::CUDA_TYPE), &format!("({})rhs[rhs_offset]", K::CUDA_TYPE))
                ),
                res.device(),
                &["binop"],
            )?;
            let kernel = res
                .device()
                .get_func(&module_name, "binop")
                .unwrap();
            let mut out_slice = unsafe {
                res.device()
                    .upgrade_device_ptr::<K>(res.ptr().ptr as u64, res.size())
            };
            let rhs_slice = unsafe {
                rhs.device()
                    .upgrade_device_ptr::<B>(rhs.ptr().ptr as u64, rhs.size())
            };
            let lhs_slice = unsafe {
                lhs.device()
                    .upgrade_device_ptr::<A>(lhs.ptr().ptr as u64, lhs.size())
            };
            let reg_info = map
                .get("binop")
                .expect("func_name not found");
            let cfg = compute_kernel_launch_config(res.device(), reg_info, res.size());
            unsafe { kernel.launch(cfg, (&mut out_slice, &lhs_slice, &rhs_slice)) }?;
            out_slice.leak();
            lhs_slice.leak();
            rhs_slice.leak();
            Ok(res)
        }
    }
}

fn extract_out<A, K, O, const CUDA_DEVICE: usize>(
    size: usize,
    res_shape: &Shape,
    out: Option<O>,
) -> anyhow::Result<_Tensor<K, Cuda, CUDA_DEVICE>>
where
    A: CommonBounds + DeviceRepr,
    K: CommonBounds + DeviceRepr,
    O: Borrow<_Tensor<K, Cuda, CUDA_DEVICE>>,
{
    let ret = if let Some(out) = out {
        if out.borrow().size() * size_of::<K>() != size * size_of::<A>() {
            return Err(InvalidOutSize(
                size * size_of::<A>(),
                out.borrow().size() * size_of::<K>(),
                Location::caller(),
            )
            .into());
        } else {
            out.borrow().static_cast::<K>()?
        }
    } else {
        _Tensor::<K, Cuda, CUDA_DEVICE>::empty(res_shape)?
    };
    Ok(ret)
}
