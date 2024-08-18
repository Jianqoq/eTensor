use rayon::iter::{ IntoParallelIterator, ParallelIterator };
use tensor_traits::CommonBounds;
use crate::tensor_base::_Tensor;
use tensor_types::into_scalar::IntoScalar;
use num::traits::MulAdd;
use tensor_types::type_promote::NormalOut;
use tensor_traits::TensorInfo;
use tensor_traits::TensorCreator;
use crate::slice::SliceOps;
use tensor_macros::match_selection;
use tensor_common::slice::Slice;

#[cfg(target_feature = "fma")]
pub fn conv2d_block_simd_parallel_unroll_i32<T>(
    img: &_Tensor<T>,
    kernels: &_Tensor<T>,
    steps: [i64; 2]
)
    -> anyhow::Result<_Tensor<T>>
    where
        T: CommonBounds + std::ops::Mul<Output = T> + std::ops::AddAssign<T> + MulAdd<Output = T>,
        T: IntoScalar<i32>
{
    use wide::i32x8;

    let img_shape = img.shape();
    let img_height = img_shape[0];
    let img_width = img_shape[1];
    let img_channels = img_shape[2];
    let kernel_shape = kernels.shape();
    let kernel_height = kernel_shape[0];
    let kernel_width = kernel_shape[1];
    let in_channels = kernel_shape[2];
    let out_channels = kernel_shape[3];
    if in_channels != img_channels {
        panic!(
            "The number of input channels in the image must be equal to the number of input channels in the kernel."
        );
    }
    let (step_width, step_height) = (steps[0], steps[1]);

    let out_height =
        <i64 as NormalOut<i64>>::_floor((img_height - kernel_height) / step_height) + 1;
    let out_width = <i64 as NormalOut<i64>>::_floor((img_width - kernel_width) / step_width) + 1;
    let output = _Tensor::<T>::zeros([out_height, out_width, out_channels])?;
    let inp = img.ptr();
    let kernel = kernels.ptr();

    let os0 = output.strides()[0]; // height
    let os1 = output.strides()[1]; // width
    let os2 = output.strides()[2]; // channels

    let is0 = img.strides()[0]; // height
    let is1 = img.strides()[1]; // width
    let is2 = img.strides()[2]; // channels

    let ks0 = kernels.strides()[0]; // kernel_height
    let ks1 = kernels.strides()[1]; // kernel_width
    let ks2 = kernels.strides()[2]; // in_channels
    let ks3 = kernels.strides()[3]; // out_channels

    let c_ob = 8;
    let c_ib = 4;
    let w_ob = 14;
    let jp_end = (out_channels + c_ob - 1) / c_ob;
    let ip_end = (in_channels + c_ib - 1) / c_ib;
    let kp_end = (out_width + w_ob - 1) / w_ob;
    (0..jp_end).into_par_iter().for_each_init(
        || output.ptr(),
        |out, jp| {
            let mut res_vectors = [i32x8::splat(0i32); 14];
            let mut res_ptrs = [0 as *mut i32; 14];
            for ip in 0..ip_end {
                for l in 0..out_height {
                    for kp in 0..kp_end {
                        for k in 0..14 {
                            let _k = kp * w_ob + k;
                            let res_ptr = &mut out[jp * c_ob * os2 + _k * os1 + l * os0]; // prettier-ignore
                            let res_vec = unsafe { std::slice::from_raw_parts_mut(res_ptr, 8) }; // prettier-ignore
                            res_vectors[k as usize]
                                .as_array_mut()
                                .copy_from_slice(unsafe {
                                    std::mem::transmute::<&[T], &[i32]>(res_vec)
                                });
                            res_ptrs[k as usize] = res_vec.as_mut_ptr() as *mut i32;
                        }
                        for n in 0..kernel_height {
                            for m in 0..kernel_width {
                                let mut scalar_vec = i32x8::splat(0i32);
                                for i in 0..c_ib {
                                    let _i = ip * c_ib + i;
                                    let kernel_ptr = &kernel[_i * ks2 + jp * c_ob * ks3 + m * ks1 + n * ks0] as *const T; // prettier-ignore
                                    let kernel_vec = unsafe { std::slice::from_raw_parts(kernel_ptr, 8) }; // prettier-ignore
                                    let kernel_vector = unsafe { i32x8::from(std::mem::transmute::<&[T], &[i32]>(kernel_vec)) }; // prettier-ignore
                                    let _kernel_vector_arr = kernel_vector.to_array();
                                    for k in 0..14 {
                                        let res_vector = &mut res_vectors[k as usize];

                                        let i_val = inp[_i * is2 + ((kp * w_ob + k) * step_width + m) * is1 + (l * step_height + n) * is0]; // prettier-ignore
                                        scalar_vec
                                            .as_array_mut()
                                            .copy_from_slice(&[i_val.into_scalar(); 8]);
                                        let _scalar_arr = scalar_vec.to_array();
                                        let res = kernel_vector * scalar_vec + *res_vector; // prettier-ignore
                                        res_vector
                                            .as_array_mut()
                                            .copy_from_slice(res.as_array_ref());
                                    }
                                }
                            }
                        }
                        for k in 0..14 {
                            let res_vector = &res_vectors[k as usize].as_array_ref();
                            let res_ptr = res_ptrs[k as usize];
                            unsafe {
                                std::ptr::copy_nonoverlapping(
                                    res_vector.as_ptr() as *const i32,
                                    res_ptr as *mut i32,
                                    8
                                );
                            }
                        }
                    }
                }
            }
        }
    );
    Ok(output)
}

#[cfg(target_feature = "fma")]
pub fn conv2d_block_simd_parallel_unroll_f32<T>(
    img: &_Tensor<T>,
    kernels: &_Tensor<T>,
    steps: [i64; 2]
)
    -> anyhow::Result<_Tensor<T>>
    where
        T: CommonBounds + std::ops::Mul<Output = T> + std::ops::AddAssign<T> + MulAdd<Output = T>,
        T: IntoScalar<f32>
{
    use wide::f32x8;

    let img_shape = img.shape();
    let img_height = img_shape[0];
    let img_width = img_shape[1];
    let img_channels = img_shape[2];
    let kernel_shape = kernels.shape();
    let kernel_height = kernel_shape[0];
    let kernel_width = kernel_shape[1];
    let in_channels = kernel_shape[2];
    let out_channels = kernel_shape[3];
    if in_channels != img_channels {
        panic!(
            "The number of input channels in the image must be equal to the number of input channels in the kernel."
        );
    }
    let (step_width, step_height) = (steps[0], steps[1]);

    let out_height =
        <i64 as NormalOut<i64>>::_floor((img_height - kernel_height) / step_height) + 1;
    let out_width = <i64 as NormalOut<i64>>::_floor((img_width - kernel_width) / step_width) + 1;
    let output = _Tensor::<T>::zeros([out_height, out_width, out_channels])?;
    let inp = img.ptr();
    let kernel = kernels.ptr();

    let os0 = output.strides()[0]; // height
    let os1 = output.strides()[1]; // width
    let os2 = output.strides()[2]; // channels

    let is0 = img.strides()[0]; // height
    let is1 = img.strides()[1]; // width
    let is2 = img.strides()[2]; // channels

    let ks0 = kernels.strides()[0]; // kernel_height
    let ks1 = kernels.strides()[1]; // kernel_width
    let ks2 = kernels.strides()[2]; // in_channels
    let ks3 = kernels.strides()[3]; // out_channels

    let c_ob = 8;
    let w_ob = 14;
    let jp_end = (out_channels + c_ob - 1) / c_ob;
    let kp_end = (out_width + w_ob - 1) / w_ob;
    (0..jp_end).into_par_iter().for_each_init(
        || output.ptr(),
        |out, jp| {
            let mut res_vectors = [f32x8::splat(0f32); 14];
            let mut res_ptrs = [0 as *mut f32; 14];
            let mut scalar_vec = f32x8::splat(0f32);
            let mut kernel_vector = f32x8::splat(0f32);
            for l in 0..out_height {
                for kp in 0..kp_end {
                    for k in 0..14 {
                        let _k = kp * w_ob + k;
                        let res_ptr = &mut out[jp * c_ob * os2 + _k * os1 + l * os0]; // prettier-ignore
                        let res_vec = unsafe { std::slice::from_raw_parts_mut(res_ptr, 8) }; // prettier-ignore
                        res_vectors[k as usize]
                            .as_array_mut()
                            .copy_from_slice(unsafe {
                                std::mem::transmute::<&[T], &[f32]>(res_vec)
                            });
                        res_ptrs[k as usize] = res_vec.as_mut_ptr() as *mut f32;
                    }
                    for n in 0..kernel_height {
                        for m in 0..kernel_width {
                            for i in 0..in_channels {
                                let kernel_ptr = &kernel[i * ks2 + jp * c_ob * ks3 + m * ks1 + n * ks0] as *const T; // prettier-ignore
                                kernel_vector
                                    .as_array_mut()
                                    .copy_from_slice(unsafe {
                                        std::mem::transmute::<&[T], &[f32]>(
                                            std::slice::from_raw_parts(kernel_ptr, 8)
                                        )
                                    });
                                for k in 0..14 {
                                    let res_vector = &mut res_vectors[k as usize];

                                    let i_val = inp[i * is2 + ((kp * w_ob + k) * step_width + m) * is1 + (l * step_height + n) * is0]; // prettier-ignore
                                    scalar_vec
                                        .as_array_mut()
                                        .copy_from_slice(&[i_val.into_scalar(); 8]);
                                    *res_vector += kernel_vector * scalar_vec;
                                }
                            }
                        }
                    }
                    for k in 0..14 {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                res_vectors[k].as_array_ref().as_ptr() as *const f32,
                                res_ptrs[k] as *mut f32,
                                8
                            );
                        }
                    }
                }
            }
        }
    );
    Ok(output)
}

#[cfg(target_feature = "fma")]
pub fn conv2d_no_group<
    T,
    VEC,
    const DVSB_REGCNT: bool,
    const DVSB_VECSIZE: bool,
    const REGCNT: usize,
    const PAD: bool
    >(
    img: &_Tensor<T>,
    kernels: &_Tensor<T>,
    steps: [i64; 2],
    padding: [(i64, i64); 2],
    dilation: [i64; 2]
)
    -> anyhow::Result<_Tensor<T>>
    where T: CommonBounds, VEC: Init<T> + Copy + VecTrait<T> + VecSize
{
    use likely_stable::likely;
    use tensor_common::{ pointer::Pointer, slice };

    let img_shape = img.shape();
    let img_height = img_shape[0];
    let img_width = img_shape[1];
    let img_channels = img_shape[2];
    let kernel_shape = kernels.shape();
    let kernel_height = kernel_shape[0];
    let kernel_width = kernel_shape[1];
    let in_channels = kernel_shape[2];
    let out_channels = kernel_shape[3];
    if in_channels != img_channels {
        panic!(
            "The number of input channels in the image must be equal to the number of input channels in the kernel."
        );
    }
    let (step_width, step_height) = (steps[0], steps[1]);
    let ((pw_start, pw_end), (ph_start, ph_end)) = (padding[0], padding[1]);
    let (dw, dh) = (dilation[0], dilation[1]);

    let out_height =
        (img_height + ph_start + ph_end - dh * (kernel_height - 1) - 1) / step_height + 1; // prettier-ignore
    let out_width = (img_width + pw_start + pw_end - dw * (kernel_width - 1) - 1) / step_width + 1; // prettier-ignore
    let img = if PAD {
        let img_padded = _Tensor::<T>::zeros([
            img_height + ph_start + ph_end,
            img_width + pw_start + pw_end,
            img_channels,
        ])?;
        let he = img_height + ph_start;
        let we = img_width + pw_start;
        let mut slice = slice!(img_padded[ph_start:he, pw_start:we, :])?;
        slice.assign(&img);
        img_padded
    } else {
        img.clone()
    };
    let output = _Tensor::<T>::zeros([out_height, out_width, out_channels])?;
    let inp = img.ptr();
    let kernel = kernels.ptr();

    let os0 = output.strides()[0]; // height
    let os1 = output.strides()[1]; // width
    let os2 = output.strides()[2]; // channels

    let is0 = img.strides()[0]; // height
    let is1 = img.strides()[1]; // width
    let is2 = img.strides()[2]; // channels

    let ks0 = kernels.strides()[0]; // kernel_height
    let ks1 = kernels.strides()[1]; // kernel_width
    let ks2 = kernels.strides()[2]; // in_channels
    let ks3 = kernels.strides()[3]; // out_channels

    let c_ob = VEC::SIZE as i64;
    let w_ob = REGCNT as i64;
    let jp_end = (out_channels + c_ob - 1) / c_ob;
    let kp_end = (out_width + w_ob - 1) / w_ob;
    if DVSB_REGCNT {
        assert_eq!(kp_end * w_ob, out_width);
    }
    if DVSB_VECSIZE {
        assert_eq!(jp_end * c_ob, out_channels);
    }
    (0..jp_end).into_par_iter().for_each_init(
        || output.ptr(),
        |out, jp| {
            let mut res_vectors = [VEC::splat(T::ZERO); REGCNT];
            let mut res_ptrs = [0 as *mut T; REGCNT];
            let mut kernel_vector = VEC::splat(T::ZERO);
            let mut stop;
            for l in 0..out_height {
                for kp in 0..kp_end {
                    stop = 0;
                    for k in 0..w_ob {
                        let _k = kp * w_ob + k;
                        let load_register = |out: &mut Pointer<T>, vec: &mut [VEC; REGCNT], ptrs: &mut [*mut T; REGCNT], stop: &mut i64| {
                            let res_ptr = &mut out[jp * c_ob * os2 + _k * os1 + l * os0]; // prettier-ignore
                            let res_vec = unsafe { std::slice::from_raw_parts_mut(res_ptr, VEC::SIZE) }; // prettier-ignore
                            vec[k as usize].copy_from_slice(res_vec);
                            ptrs[k as usize] = res_vec.as_mut_ptr() as *mut T;
                            *stop += 1;
                        }; // prettier-ignore
                        if DVSB_REGCNT {
                            load_register(out, &mut res_vectors, &mut res_ptrs, &mut stop);
                        } else {
                            if likely(_k < out_width) {
                                load_register(out, &mut res_vectors, &mut res_ptrs, &mut stop);
                            }
                        }
                    }
                    for n in 0..kernel_height {
                        for m in 0..kernel_width {
                            for i in 0..in_channels {
                                let kernel_ptr = &kernel[i * ks2 + jp * c_ob * ks3 + m * ks1 + n * ks0] as *const T; // prettier-ignore
                                kernel_vector.copy_from_slice(unsafe {
                                    std::slice::from_raw_parts(kernel_ptr, VEC::SIZE)
                                });
                                for k in 0..stop {
                                    let res_vector = &mut res_vectors[k as usize];
                                    let i_val = inp[i * is2 + ((kp * w_ob + k) * step_width + m * dw) * is1 + (l * step_height + n * dh) * is0]; // prettier-ignore
                                    res_vector.fma(kernel_vector, VEC::splat(i_val));
                                }
                            }
                        }
                    }
                    for k in 0..stop {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                res_vectors[k as usize].as_ptr(),
                                res_ptrs[k as usize],
                                VEC::SIZE
                            );
                        }
                    }
                }
            }
        }
    );
    Ok(output)
}

#[cfg(target_feature = "fma")]
pub fn conv2d_no_group_f32<T, const PAD: bool>(
    img: &_Tensor<T>,
    kernels: &_Tensor<T>,
    steps: [i64; 2],
    padding: [(i64, i64); 2],
    dilation: [i64; 2]
)
    -> anyhow::Result<_Tensor<T>>
    where T: CommonBounds, T: IntoScalar<f32>
{
    use likely_stable::likely;
    use tensor_common::{ pointer::Pointer, slice };
    use wide::f32x8;

    let img_shape = img.shape();
    let img_height = img_shape[0];
    let img_width = img_shape[1];
    let img_channels = img_shape[2];
    let kernel_shape = kernels.shape();
    let kernel_height = kernel_shape[0];
    let kernel_width = kernel_shape[1];
    let in_channels = kernel_shape[2];
    let out_channels = kernel_shape[3];
    if in_channels != img_channels {
        panic!(
            "The number of input channels in the image must be equal to the number of input channels in the kernel."
        );
    }
    let (step_width, step_height) = (steps[0], steps[1]);
    let ((pw_start, pw_end), (ph_start, ph_end)) = (padding[0], padding[1]);
    let (dw, dh) = (dilation[0], dilation[1]);

    let out_height =
        (img_height + ph_start + ph_end - dh * (kernel_height - 1) - 1) / step_height + 1; // prettier-ignore
    let out_width = (img_width + pw_start + pw_end - dw * (kernel_width - 1) - 1) / step_width + 1; // prettier-ignore
    let img = if PAD {
        let img_padded = _Tensor::<T>::zeros([
            img_height + ph_start + ph_end,
            img_width + pw_start + pw_end,
            img_channels,
        ])?;
        let he = img_height + ph_start;
        let we = img_width + pw_start;
        let mut slice = slice!(img_padded[ph_start:he, pw_start:we, :])?;
        slice.assign(&img);
        img_padded
    } else {
        img.clone()
    };
    let output = _Tensor::<T>::zeros([out_height, out_width, out_channels])?;
    let inp = img.ptr();
    let kernel = kernels.ptr();

    let os0 = output.strides()[0]; // height
    let os1 = output.strides()[1]; // width
    let os2 = output.strides()[2]; // channels

    let is0 = img.strides()[0]; // height
    let is1 = img.strides()[1]; // width
    let is2 = img.strides()[2]; // channels

    let ks0 = kernels.strides()[0]; // kernel_height
    let ks1 = kernels.strides()[1]; // kernel_width
    let ks2 = kernels.strides()[2]; // in_channels
    let ks3 = kernels.strides()[3]; // out_channels

    let c_ob = 8;
    let w_ob = 14;
    let jp_end = (out_channels + c_ob - 1) / c_ob;
    let kp_end = (out_width + w_ob - 1) / w_ob;
    (0..jp_end).into_par_iter().for_each_init(
        || output.ptr(),
        |out, jp| {
            let mut res_vectors = [f32x8::splat(0f32); 14];
            let mut res_ptrs = [0 as *mut T; 14];
            let mut kernel_vector = f32x8::splat(0f32);
            let mut stop;
            for l in 0..out_height {
                for kp in 0..kp_end {
                    stop = 0;
                    for k in 0..w_ob {
                        let _k = kp * w_ob + k;
                        let load_register = |out: &mut Pointer<T>, vec: &mut [f32x8; 14], ptrs: &mut [*mut T; 14], stop: &mut i64| {
                            let res_ptr = &mut out[jp * c_ob * os2 + _k * os1 + l * os0]; // prettier-ignore
                            let res_vec = unsafe { std::slice::from_raw_parts_mut(res_ptr, 8) }; // prettier-ignore
                            vec[k as usize].copy_from_slice(
                                unsafe { std::mem::transmute::<&[T], &[f32]>(res_vec) }
                            );
                            ptrs[k as usize] = res_vec.as_mut_ptr() as *mut T;
                            *stop += 1;
                        }; // prettier-ignore
                        if likely(_k < out_width) {
                            load_register(out, &mut res_vectors, &mut res_ptrs, &mut stop);
                        }
                    }
                    for n in 0..kernel_height {
                        for m in 0..kernel_width {
                            for i in 0..in_channels {
                                let kernel_ptr = &kernel[i * ks2 + jp * c_ob * ks3 + m * ks1 + n * ks0] as *const T; // prettier-ignore
                                kernel_vector.copy_from_slice(unsafe {
                                    std::slice::from_raw_parts(kernel_ptr as *const f32, 8)
                                });
                                for k in 0..stop {
                                    let res_vector = &mut res_vectors[k as usize];
                                    let i_val = inp[i * is2 + ((kp * w_ob + k) * step_width + m * dw) * is1 + (l * step_height + n * dh) * is0]; // prettier-ignore
                                    *res_vector +=
                                        kernel_vector * f32x8::splat(i_val.into_scalar());
                                }
                            }
                        }
                    }
                    for k in 0..stop {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                res_vectors[k as usize].as_ptr(),
                                res_ptrs[k as usize] as *mut f32,
                                8
                            );
                        }
                    }
                }
            }
        }
    );
    Ok(output)
}

use crate::ops::cpu::vector::traits::Init;
use crate::ops::cpu::vector::traits::VecTrait;
use crate::ops::cpu::vector::traits::VecSize;

#[cfg(all(target_feature = "fma", target_feature = "avx2"))]
pub fn conv2d_group<
    T,
    VEC,
    const DVSB_REGCNT: bool,
    const DVSB_VECSIZE: bool,
    const REGCNT: usize,
    const PAD: bool
    >(
    img: &_Tensor<T>,
    kernels: &_Tensor<T>,
    steps: [i64; 2],
    padding: [(i64, i64); 2],
    dilation: [i64; 2],
    groups: i64
)
    -> anyhow::Result<_Tensor<T>>
    where T: CommonBounds, VEC: Init<T> + Copy + VecTrait<T> + VecSize
{
    use likely_stable::likely;
    use tensor_common::{ pointer::Pointer, slice };

    let img_shape = img.shape();
    let img_height = img_shape[0];
    let img_width = img_shape[1];
    let img_channels = img_shape[2];
    let kernel_shape = kernels.shape();
    let kernel_height = kernel_shape[0];
    let kernel_width = kernel_shape[1];
    let in_channels = kernel_shape[2];
    let out_channels = kernel_shape[3];
    if in_channels != img_channels {
        panic!(
            "The number of input channels in the image must be equal to the number of input channels in the kernel."
        );
    }
    if in_channels % groups != 0 || out_channels % groups != 0 {
        panic!(
            "The number of input and output channels must be divisible by the number of groups."
        );
    }
    let kernels_per_group = out_channels / groups;
    let channels_per_group = in_channels / groups;
    let (step_width, step_height) = (steps[0], steps[1]);
    let ((pw_start, pw_end), (ph_start, ph_end)) = (padding[0], padding[1]);
    let (dw, dh) = (dilation[0], dilation[1]);

    let out_height =
        (img_height + ph_start + ph_end - dh * (kernel_height - 1) - 1) / step_height + 1; // prettier-ignore
    let out_width = (img_width + pw_start + pw_end - dw * (kernel_width - 1) - 1) / step_width + 1; // prettier-ignore
    let img = if PAD {
        let img_padded = _Tensor::<T>::zeros([
            img_height + ph_start + ph_end,
            img_width + pw_start + pw_end,
            img_channels,
        ])?;
        let he = img_height + ph_start;
        let we = img_width + pw_start;
        let mut slice = slice!(img_padded[ph_start:he, pw_start:we, :])?;
        slice.assign(&img);
        img_padded
    } else {
        img.clone()
    };
    let output = _Tensor::<T>::zeros([out_height, out_width, out_channels])?;
    let inp = img.ptr();
    let kernel = kernels.ptr();

    let os0 = output.strides()[0]; // height
    let os1 = output.strides()[1]; // width
    let os2 = output.strides()[2]; // channels

    let is0 = img.strides()[0]; // height
    let is1 = img.strides()[1]; // width
    let is2 = img.strides()[2]; // channels

    let ks0 = kernels.strides()[0]; // kernel_height
    let ks1 = kernels.strides()[1]; // kernel_width
    let ks2 = kernels.strides()[2]; // in_channels
    let ks3 = kernels.strides()[3]; // out_channels

    let c_ob = VEC::SIZE as i64;
    let w_ob = REGCNT as i64;
    let kp_end = (out_width + w_ob - 1) / w_ob;
    if DVSB_REGCNT {
        assert_eq!(kp_end * w_ob, out_width);
    }
    let jp_end = (kernels_per_group + c_ob - 1) / c_ob;
    if DVSB_VECSIZE {
        assert_eq!(jp_end * c_ob, kernels_per_group);
    }
    (0..groups).into_par_iter().for_each_init(
        || output.ptr(),
        |out, g| {
            let mut res_vectors = [VEC::splat(T::ZERO); REGCNT];
            let mut res_ptrs = [0 as *mut T; REGCNT];
            let mut kernel_vector = VEC::splat(T::ZERO);
            for jp in 0..jp_end {
                for l in 0..out_height {
                    for kp in 0..kp_end {
                        let mut stop = 0;
                        for k in 0..w_ob {
                            let _k = kp * w_ob + k;
                            let load_to_reg = |out: &mut Pointer<T>, vec: &mut [VEC; REGCNT], ptrs: &mut [*mut T; REGCNT], stop: &mut i64| {
                                let res_ptr = &mut out[(g * kernels_per_group + jp * c_ob) * os2 + _k * os1 + l * os0]; // prettier-ignore
                                let res_vec = unsafe { std::slice::from_raw_parts_mut(res_ptr, VEC::SIZE) }; // prettier-ignore
                                vec[k as usize].copy_from_slice(res_vec);
                                ptrs[k as usize] = res_vec.as_mut_ptr();
                                *stop += 1;
                            }; // prettier-ignore
                            if DVSB_REGCNT {
                                load_to_reg(out, &mut res_vectors, &mut res_ptrs, &mut stop);
                            } else {
                                if likely(_k < out_width) {
                                    load_to_reg(out, &mut res_vectors, &mut res_ptrs, &mut stop);
                                }
                            }
                        }
                        for n in 0..kernel_height {
                            for m in 0..kernel_width {
                                for i in 0..channels_per_group {
                                    let kernel_ptr = &kernel[i * ks2 + (g * kernels_per_group + jp * c_ob) * ks3 + m * ks1 + n * ks0] as *const T; // prettier-ignore
                                    kernel_vector.copy_from_slice(unsafe {
                                        std::slice::from_raw_parts(kernel_ptr, VEC::SIZE)
                                    });
                                    for k in 0..stop {
                                        let res_vector = &mut res_vectors[k as usize];
                                        let i_val = inp[(g * channels_per_group + i) * is2 + ((kp * w_ob + k) * step_width + m * dw) * is1 + (l * step_height + n * dh) * is0]; // prettier-ignore
                                        res_vector.fma(kernel_vector, VEC::splat(i_val));
                                    }
                                }
                            }
                        }
                        for k in 0..stop {
                            unsafe {
                                std::ptr::copy_nonoverlapping(
                                    res_vectors[k as usize].as_ptr(),
                                    res_ptrs[k as usize],
                                    VEC::SIZE
                                );
                            }
                        }
                    }
                }
            }
        }
    );
    Ok(output)
}

#[cfg(target_feature = "fma")]
pub fn conv2d_ex_f32<T, VEC, const PAD: bool, const REGCNT: usize>(
    img: &_Tensor<T>,
    kernels: &_Tensor<T>,
    steps: [i64; 2],
    padding: [(i64, i64); 2],
    dilation: [i64; 2]
)
    -> anyhow::Result<_Tensor<T>>
    where
        T: CommonBounds + std::ops::Mul<Output = T> + std::ops::AddAssign<T> + MulAdd<Output = T> + IntoScalar<f32>,
        VEC: VecSize
{
    use tensor_common::slice;
    use wide::f32x8;

    let img_shape = img.shape();
    let img_height = img_shape[0];
    let img_width = img_shape[1];
    let img_channels = img_shape[2];
    let kernel_shape = kernels.shape();
    let kernel_height = kernel_shape[0];
    let kernel_width = kernel_shape[1];
    let in_channels = kernel_shape[2];
    let out_channels = kernel_shape[3];
    if in_channels != img_channels {
        panic!(
            "The number of input channels in the image must be equal to the number of input channels in the kernel."
        );
    }
    let (step_width, step_height) = (steps[0], steps[1]);
    let ((pw_start, pw_end), (ph_start, ph_end)) = (padding[0], padding[1]);
    let (dw, dh) = (dilation[0], dilation[1]);

    let out_height =
        (img_height + ph_start + ph_end - dh * (kernel_height - 1) - 1) / step_height + 1;
    let out_width = (img_width + pw_start + pw_end - dw * (kernel_width - 1) - 1) / step_width + 1;
    let img = if PAD {
        let img_padded = _Tensor::<T>::zeros([
            img_height + ph_start + ph_end,
            img_width + pw_start + pw_end,
            img_channels,
        ])?;
        let he = img_height + ph_start;
        let we = img_width + pw_start;
        let mut slice = slice!(img_padded[ph_start:he, pw_start:we, :])?;
        slice.assign(&img);
        img_padded
    } else {
        img.clone()
    };
    let output = _Tensor::<T>::zeros([out_height, out_width, out_channels])?;
    let out = output.ptr();
    let inp = img.ptr();
    let kernel = kernels.ptr();

    let os0 = output.strides()[0]; // height
    let os1 = output.strides()[1]; // width
    let os2 = output.strides()[2]; // channels

    let is0 = img.strides()[0]; // height
    let is1 = img.strides()[1]; // width
    let is2 = img.strides()[2]; // channels

    let ks0 = kernels.strides()[0]; // kernel_height
    let ks1 = kernels.strides()[1]; // kernel_width
    let ks2 = kernels.strides()[2]; // in_channels
    let ks3 = kernels.strides()[3]; // out_channels

    let oc_r8 = out_channels % (VEC::SIZE as i64);
    if oc_r8 > 0 {
        let o_n = out_channels / (VEC::SIZE as i64);
        let ow_r14 = out_width % (REGCNT as i64);
        if ow_r14 > 0 {
            let ow_n = out_width / (REGCNT as i64);
            (0..o_n).into_par_iter().for_each_init(||out, |out, jp|{
                let mut res_vectors = [f32x8::splat(0.0); REGCNT];
                let mut res_vectors_heap = vec![f32x8::splat(0.0); ow_r14 as usize];
                let mut res_ptrs = [0 as *mut T; REGCNT];
                let mut res_ptrs_heap = vec![0 as *mut T; ow_r14 as usize];
                let mut kernel_vector = f32x8::splat(0.0);
                for l in 0..out_height {
                    for kp in 0..ow_n {
                        for k in 0..REGCNT as i64 {
                            let _k = kp * (REGCNT as i64) + k;
                            let res_ptr = &mut out[jp * VEC::SIZE as i64 * os2 + _k * os1 + l * os0]; // prettier-ignore
                            let res_vec = unsafe { std::slice::from_raw_parts_mut(res_ptr, VEC::SIZE) }; // prettier-ignore
                            res_vectors[k as usize].copy_from_slice(
                                unsafe {
                                    std::mem::transmute::<&[T], &[f32]>(res_vec)
                                }
                            );
                            res_ptrs[k as usize] = res_vec.as_mut_ptr() as *mut T;
                        }
                        for n in 0..kernel_height {
                            for m in 0..kernel_width {
                                for i in 0..in_channels {
                                    let kernel_ptr = &kernel[i * ks2 + jp * VEC::SIZE as i64 * ks3 + m * ks1 + n * ks0] as *const T;
                                    kernel_vector.copy_from_slice(unsafe {
                                        std::slice::from_raw_parts(
                                            kernel_ptr as *const f32,
                                            VEC::SIZE)
                                    });
                                    for k in 0..REGCNT as i64 {
                                        let res_vector = &mut res_vectors[k as usize];
                                        let i_val = inp[i * is2 + ((kp * REGCNT as i64 + k) * step_width + m * dw) * is1 + (l * step_height + n * dh) * is0];
                                        res_vector.fma(kernel_vector, f32x8::splat(i_val.into_scalar()));
                                    }
                                }
                            }
                        }
                        for k in 0..REGCNT as i64 {
                            unsafe {
                                std::ptr::copy_nonoverlapping(
                                    res_vectors[k as usize].as_ptr(),
                                    res_ptrs[k as usize] as *mut f32,
                                    VEC::SIZE
                                );
                            }
                        }
                    }
                    for kp in ow_n..ow_n + 1 {
                        for k in 0..ow_r14 {
                            let _k = kp * (REGCNT as i64) + k;
                            let res_ptr = &mut out[jp * VEC::SIZE as i64 * os2 + _k * os1 + l * os0];
                            let res_vec = unsafe { std::slice::from_raw_parts_mut(res_ptr, VEC::SIZE) };
                            res_vectors_heap[k as usize].copy_from_slice(
                                unsafe {
                                    std::mem::transmute::<&[T], &[f32]>(res_vec)
                                }
                            );
                            res_ptrs_heap[k as usize] = res_vec.as_mut_ptr() as *mut T;
                        }
                        for n in 0..kernel_height {
                            for m in 0..kernel_width {
                                for i in 0..in_channels {
                                    let kernel_ptr = &kernel[i * ks2 + jp * VEC::SIZE as i64 * ks3 + m * ks1 + n * ks0] as *const T;
                                    kernel_vector.copy_from_slice(unsafe {
                                        std::slice::from_raw_parts(
                                            kernel_ptr as *const f32
                                            , VEC::SIZE)
                                    });
                                    for k in 0..ow_r14 as i64 {
                                        let res_vector = &mut res_vectors_heap[k as usize];
                                        let i_val = inp[i * is2 + ((kp * REGCNT as i64 + k) * step_width + m * dw) * is1 + (l * step_height + n * dh) * is0];
                                        res_vector.fma(kernel_vector, f32x8::splat(i_val.into_scalar()));
                                    }
                                }
                            }
                        }
                        for k in 0..ow_r14 {
                            unsafe {
                                std::ptr::copy_nonoverlapping(
                                    res_vectors_heap[k as usize].as_ptr(),
                                    res_ptrs_heap[k as usize] as *mut f32,
                                    VEC::SIZE
                                );
                            }
                        }
                    }
                }
            }); // prettier-ignore
            for jp in o_n..o_n + 1 {
                (0..out_height).into_par_iter().for_each_init(
                    || out,
                    |out, l| {
                        let mut res_vectors = vec![vec![T::ZERO; oc_r8 as usize]; REGCNT];
                        let mut res_vectors_heap =
                            vec![vec![T::ZERO; oc_r8 as usize]; ow_r14 as usize];
                        let mut res_ptrs = [0 as *mut T; REGCNT];
                        let mut res_ptrs_heap = vec![0 as *mut T; ow_r14 as usize];
                        let mut kernel_vector = vec![T::ZERO; oc_r8 as usize];
                        for kp in 0..ow_n {
                            for k in 0..REGCNT as i64 {
                                let _k = kp * (REGCNT as i64) + k;
                                let res_ptr = &mut out[jp * VEC::SIZE as i64 * os2 + _k * os1 + l * os0]; // prettier-ignore
                                let res_vec = unsafe { std::slice::from_raw_parts_mut(res_ptr, oc_r8 as usize) }; // prettier-ignore
                                res_vectors[k as usize].copy_from_slice(res_vec);
                                res_ptrs[k as usize] = res_vec.as_mut_ptr() as *mut T;
                            }
                            for n in 0..kernel_height {
                                for m in 0..kernel_width {
                                    for i in 0..in_channels {
                                        let kernel_ptr = &kernel[i * ks2 + jp * VEC::SIZE as i64 * ks3 + m * ks1 + n * ks0] as *const T; // prettier-ignore
                                        kernel_vector.copy_from_slice(unsafe {
                                            std::slice::from_raw_parts(kernel_ptr, oc_r8 as usize)
                                        });
                                        for k in 0..REGCNT as i64 {
                                            let res_vector = &mut res_vectors[k as usize];
                                            let i_val = inp[i * is2 + ((kp * REGCNT as i64 + k) * step_width + m * dw) * is1 + (l * step_height + n * dh) * is0]; // prettier-ignore
                                            res_vector
                                                .iter_mut()
                                                .zip(kernel_vector.iter())
                                                .for_each(|(res, ker)| {
                                                    *res += i_val * *ker;
                                                });
                                        }
                                    }
                                }
                            }
                            for k in 0..REGCNT as i64 {
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        res_vectors[k as usize].as_ptr(),
                                        res_ptrs[k as usize],
                                        oc_r8 as usize
                                    );
                                }
                            }
                        }
                        let now = std::time::Instant::now();
                        for kp in ow_n..ow_n + 1 {
                            for k in 0..ow_r14 {
                                let _k = kp * (REGCNT as i64) + k;
                                let res_ptr = &mut out[jp * VEC::SIZE as i64 * os2 + _k * os1 + l * os0]; // prettier-ignore
                                let res_vec = unsafe { std::slice::from_raw_parts_mut(res_ptr, oc_r8 as usize) }; // prettier-ignore
                                res_vectors_heap[k as usize].copy_from_slice(res_vec);
                                res_ptrs_heap[k as usize] = res_vec.as_mut_ptr() as *mut T;
                            }
                            for n in 0..kernel_height {
                                for m in 0..kernel_width {
                                    for i in 0..in_channels {
                                        let kernel_ptr = &kernel[i * ks2 + jp * VEC::SIZE as i64 * ks3 + m * ks1 + n * ks0] as *const T; // prettier-ignore
                                        kernel_vector.copy_from_slice(unsafe {
                                            std::slice::from_raw_parts(kernel_ptr, oc_r8 as usize)
                                        });
                                        for k in 0..ow_r14 as i64 {
                                            let res_vector = &mut res_vectors_heap[k as usize];
                                            let i_val = inp[i * is2 + ((kp * REGCNT as i64 + k) * step_width + m * dw) * is1 + (l * step_height + n * dh) * is0]; // prettier-ignore
                                            res_vector
                                                .iter_mut()
                                                .zip(kernel_vector.iter())
                                                .for_each(|(res, ker)| {
                                                    *res += i_val * *ker;
                                                });
                                        }
                                    }
                                }
                            }
                            for k in 0..ow_r14 {
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        res_vectors_heap[k as usize].as_ptr(),
                                        res_ptrs_heap[k as usize],
                                        oc_r8 as usize
                                    );
                                }
                            }
                        }
                        println!("{:?}", now.elapsed());
                    }
                );
            }
        } else {
            let kp_end = out_width / (REGCNT as i64);
            let o_n = out_channels / (VEC::SIZE as i64);
            (0..o_n).into_par_iter().for_each_init(||out, |out, jp|{
                let mut res_vectors = [f32x8::splat(0.0); REGCNT];
                let mut res_ptrs = [0 as *mut T; REGCNT];
                let mut kernel_vector = f32x8::splat(0.0);
                for l in 0..out_height {
                    for kp in 0..kp_end {
                        for k in 0..REGCNT as i64 {
                            let _k = kp * (REGCNT as i64) + k;
                            let res_ptr = &mut out[jp * VEC::SIZE as i64 * os2 + _k * os1 + l * os0]; // prettier-ignore
                            let res_vec = unsafe { std::slice::from_raw_parts_mut(res_ptr, VEC::SIZE) }; // prettier-ignore
                            res_vectors[k as usize].copy_from_slice(
                                unsafe {
                                    std::mem::transmute::<&[T], &[f32]>(res_vec)
                                }
                            );
                            res_ptrs[k as usize] = res_vec.as_mut_ptr() as *mut T;
                        }
                        for n in 0..kernel_height {
                            for m in 0..kernel_width {
                                for i in 0..in_channels {
                                    let kernel_ptr = &kernel[i * ks2 + jp * VEC::SIZE as i64 * ks3 + m * ks1 + n * ks0] as *const T; // prettier-ignore
                                    kernel_vector.copy_from_slice(unsafe {
                                        std::slice::from_raw_parts(
                                            kernel_ptr as *const f32
                                            , VEC::SIZE)
                                    });
                                    for k in 0..REGCNT as i64 {
                                        let res_vector = &mut res_vectors[k as usize];
                                        let i_val = inp[i * is2 + ((kp * REGCNT as i64 + k) * step_width + m * dw) * is1 + (l * step_height + n * dh) * is0]; // prettier-ignore
                                        res_vector.fma(kernel_vector, f32x8::splat(i_val.into_scalar()));
                                    }
                                }
                            }
                        }
                        for k in 0..REGCNT {
                            unsafe {
                                std::ptr::copy_nonoverlapping(
                                    res_vectors[k].as_ptr(),
                                    res_ptrs[k] as *mut f32,
                                    VEC::SIZE
                                );
                            }
                        }
                    }
                }
            }); // prettier-ignore
            for jp in o_n..o_n + 1 {
                (0..out_height).into_par_iter().for_each_init(
                    || out,
                    |out, l| {
                        let mut res_vectors = vec![vec![T::ZERO; oc_r8 as usize]; REGCNT];
                        let mut res_ptrs = [0 as *mut T; REGCNT];
                        let mut kernel_vector = vec![T::ZERO; oc_r8 as usize];
                        for kp in 0..kp_end {
                            for k in 0..REGCNT as i64 {
                                let _k = kp * (REGCNT as i64) + k;
                                let res_ptr = &mut out[jp * VEC::SIZE as i64 * os2 + _k * os1 + l * os0]; // prettier-ignore
                                let res_vec = unsafe { std::slice::from_raw_parts_mut(res_ptr, oc_r8 as usize) }; // prettier-ignore
                                res_vectors[k as usize].copy_from_slice(res_vec);
                                res_ptrs[k as usize] = res_vec.as_mut_ptr() as *mut T;
                            }
                            for n in 0..kernel_height {
                                for m in 0..kernel_width {
                                    for i in 0..in_channels {
                                        let kernel_ptr = &kernel[i * ks2 + jp * VEC::SIZE as i64 * ks3 + m * ks1 + n * ks0] as *const T; // prettier-ignore
                                        kernel_vector.copy_from_slice(unsafe {
                                            std::slice::from_raw_parts(kernel_ptr, oc_r8 as usize)
                                        });
                                        for k in 0..REGCNT as i64 {
                                            let res_vector = &mut res_vectors[k as usize];
                                            let i_val = inp[i * is2 + ((kp * REGCNT as i64 + k) * step_width + m * dw) * is1 + (l * step_height + n * dh) * is0]; // prettier-ignore
                                            res_vector.iter_mut().enumerate().for_each(|(idx, val)| {
                                            *val += unsafe { *kernel_vector.as_ptr().wrapping_add(idx) } * i_val;
                                        }); // prettier-ignore
                                        }
                                    }
                                }
                            }
                            for k in 0..REGCNT {
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        res_vectors[k].as_ptr(),
                                        res_ptrs[k],
                                        oc_r8 as usize
                                    );
                                }
                            }
                        }
                    }
                );
            }
        }
    } else {
        let ow_r14 = out_width % (REGCNT as i64);
        if ow_r14 > 0 {
            let jp_end = out_channels / (VEC::SIZE as i64);
            let kp_end = out_width / (REGCNT as i64);
            (0..jp_end).into_par_iter().for_each_init(
                || out,
                |out, jp| {
                    let mut res_vectors = [f32x8::splat(0.0); REGCNT];
                    let mut res_vectors_heap = vec![f32x8::splat(0.); ow_r14 as usize];
                    let mut res_ptrs = [0 as *mut T; REGCNT];
                    let mut res_ptrs_heap = vec![0 as *mut T; ow_r14 as usize];
                    let mut kernel_vector = f32x8::splat(0.0);
                    let mut now = std::time::Duration::new(0, 0);
                    let mut now2 = std::time::Duration::new(0, 0);
                    for l in 0..out_height {
                        let tmp_now = std::time::Instant::now();
                        for kp in 0..kp_end {
                            for k in 0..REGCNT as i64 {
                                let _k = kp * (REGCNT as i64) + k;
                                let res_ptr = &mut out[jp * VEC::SIZE as i64 * os2 + _k * os1 + l * os0]; // prettier-ignore
                                let res_vec = unsafe { std::slice::from_raw_parts_mut(res_ptr, VEC::SIZE) }; // prettier-ignore
                                res_vectors[k as usize].copy_from_slice(unsafe {
                                    std::mem::transmute::<&[T], &[f32]>(res_vec)
                                });
                                res_ptrs[k as usize] = res_vec.as_mut_ptr() as *mut T;
                            }
                            for n in 0..kernel_height {
                                for m in 0..kernel_width {
                                    for i in 0..in_channels {
                                        let kernel_ptr = &kernel[i * ks2 + jp * VEC::SIZE as i64 * ks3 + m * ks1 + n * ks0] as *const T; // prettier-ignore
                                        kernel_vector.copy_from_slice(unsafe {
                                            std::slice::from_raw_parts(
                                                kernel_ptr as *const f32,
                                                VEC::SIZE
                                            )
                                        });
                                        for k in 0..REGCNT as i64 {
                                            let res_vector = &mut res_vectors[k as usize];
                                            let i_val = inp[i * is2 + ((kp * REGCNT as i64 + k) * step_width + m * dw) * is1 + (l * step_height + n * dh) * is0]; // prettier-ignore
                                            *res_vector += kernel_vector * f32x8::splat(i_val.into_scalar());
                                        }
                                    }
                                }
                            }
                            for k in 0..REGCNT as i64 {
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        res_vectors[k as usize].as_ptr(),
                                        res_ptrs[k as usize] as *mut f32,
                                        VEC::SIZE
                                    );
                                }
                            }
                        }
                        now2 += tmp_now.elapsed();
                        let now2 = std::time::Instant::now();
                        for kp in kp_end..kp_end + 1 {
                            for k in 0..ow_r14 {
                                let _k = kp * (REGCNT as i64) + k;
                                let res_ptr = &mut out[jp * VEC::SIZE as i64 * os2 + _k * os1 + l * os0]; // prettier-ignore
                                let res_vec = unsafe { std::slice::from_raw_parts_mut(res_ptr, VEC::SIZE) }; // prettier-ignore
                                res_vectors_heap[k as usize].copy_from_slice(
                                    unsafe { std::mem::transmute::<&[T], &[f32]>(res_vec) }
                                );
                                res_ptrs_heap[k as usize] = res_vec.as_mut_ptr() as *mut T;
                            }
                            for n in 0..kernel_height {
                                for m in 0..kernel_width {
                                    for i in 0..in_channels {
                                        let kernel_ptr = &kernel[i * ks2 + jp * VEC::SIZE as i64 * ks3 + m * ks1 + n * ks0] as *const T; // prettier-ignore
                                        kernel_vector.copy_from_slice(unsafe {
                                            std::slice::from_raw_parts(
                                                kernel_ptr as *const f32,
                                                VEC::SIZE)
                                        });
                                        for k in 0..ow_r14 as i64 {
                                            let res_vector = &mut res_vectors_heap[k as usize];
                                            let i_val = inp[i * is2 + ((kp * REGCNT as i64 + k) * step_width + m * dw) * is1 + (l * step_height + n * dh) * is0]; // prettier-ignore
                                            res_vector.fma(kernel_vector, f32x8::splat(i_val.into_scalar()));
                                        }
                                    }
                                }
                            }
                            for k in 0..ow_r14 {
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        res_vectors_heap[k as usize].as_ptr(),
                                        res_ptrs_heap[k as usize] as *mut f32,
                                        VEC::SIZE
                                    );
                                }
                            }
                        }
                        now += now2.elapsed();
                    }
                    println!("{:?}", now);
                    println!("{:?}", now2);
                }
            );
        } else {
            let jp_end = out_channels / (VEC::SIZE as i64);
            let kp_end = out_width / (REGCNT as i64);
            (0..jp_end).into_par_iter().for_each_init(
                || out,
                |out, jp| {
                    let mut res_vectors = [f32x8::splat(0.0); REGCNT];
                    let mut res_ptrs = [0 as *mut T; REGCNT];
                    let mut kernel_vector = f32x8::splat(0.0);
                    for l in 0..out_height {
                        for kp in 0..kp_end {
                            for k in 0..REGCNT as i64 {
                                let _k = kp * (REGCNT as i64) + k;
                                let res_ptr = &mut out[jp * VEC::SIZE as i64 * os2 + _k * os1 + l * os0]; // prettier-ignore
                                let res_vec = unsafe { std::slice::from_raw_parts_mut(res_ptr, VEC::SIZE) }; // prettier-ignore
                                res_vectors[k as usize].copy_from_slice(
                                    unsafe { std::mem::transmute::<&[T], &[f32]>(res_vec) }
                                );
                                res_ptrs[k as usize] = res_vec.as_mut_ptr() as *mut T;
                            }
                            for n in 0..kernel_height {
                                for m in 0..kernel_width {
                                    for i in 0..in_channels {
                                        let kernel_ptr = &kernel[i * ks2 + jp * VEC::SIZE as i64 * ks3 + m * ks1 + n * ks0] as *const T; // prettier-ignore
                                        kernel_vector.copy_from_slice(unsafe {
                                            std::slice::from_raw_parts(
                                                kernel_ptr as *const f32
                                                , VEC::SIZE)
                                        });
                                        for k in 0..REGCNT as i64 {
                                            let res_vector = &mut res_vectors[k as usize];
                                            let i_val = inp[i * is2 + ((kp * REGCNT as i64 + k) * step_width + m * dw) * is1 + (l * step_height + n * dh) * is0]; // prettier-ignore
                                            res_vector.fma(kernel_vector, f32x8::splat(
                                                i_val.into_scalar()
                                            ));
                                        }
                                    }
                                }
                            }
                            for k in 0..REGCNT {
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        res_vectors[k].as_ptr(),
                                        res_ptrs[k] as *mut f32,
                                        VEC::SIZE
                                    );
                                }
                            }
                        }
                    }
                }
            );
        }
    }
    Ok(output)
}
