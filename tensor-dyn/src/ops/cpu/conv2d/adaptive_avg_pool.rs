use crate::tensor_base::_Tensor;
use crate::Tensor;
use crate::REGNUM;
use rayon::prelude::*;
use tensor_common::error::base::TensorError;
use tensor_common::error::shape::ShapeError;
use tensor_traits::CommonBounds;
use tensor_traits::TensorCreator;
use tensor_traits::TensorInfo;
use tensor_types::into_scalar::IntoScalar;
use tensor_types::type_promote::FloatOutBinary;
use tensor_types::type_promote::NormalOut;
use tensor_types::vectors::traits::*;

impl<T> _Tensor<T>
where
    T: CommonBounds + IntoScalar<T> + NormalOut<Output = T> + FloatOutBinary<T, Output = T>,
    T::Vec: VecTrait<T>
        + Copy
        + Send
        + Sync
        + NormalOut<Output = T::Vec>
        + FloatOutBinary<T::Vec, Output = T::Vec>,
    bool: IntoScalar<T>,
    i64: IntoScalar<T>,
{
    /// Performs a 2D avg pooling operation on the input tensor.
    ///
    /// This method applies a 2D avg pooling operation on the tensor using the specified kernel,
    /// strides (steps), padding, and dilation factors.
    ///
    /// # Arguments
    ///
    /// * `kernels` - A reference to the tensor representing the convolution kernels (filters).
    ///   The size of the kernel tensor determines the spatial dimensions of the convolution operation.
    /// * `steps` - A 2-element array specifying the stride (step size) of the convolution along the height and width dimensions.
    /// * `padding` - A 2-element array of tuples representing the padding for the height and width dimensions.
    ///   Each tuple specifies the amount of padding added before and after the data along the respective axis.
    /// * `dilation` - A 2-element array specifying the dilation factor for the convolution along the height and width dimensions.
    ///   Dilation allows the kernel to be applied to inputs with gaps, increasing the receptive field of the kernel.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing the output tensor after applying the 2D avg pooling operation.
    #[cfg_attr(feature = "track_caller", track_caller)]
    #[inline(never)]
    pub fn adaptive_avgpool2d(
        &self,
        output_size: [i64; 2],
    ) -> std::result::Result<_Tensor<T>, TensorError> {
        let img_shape = self.shape();
        ShapeError::check_dim(4, img_shape.len())?;
        let batch = img_shape[0];
        let img_height = img_shape[1];
        let img_width = img_shape[2];
        let in_channels = img_shape[3];

        let out_height = output_size[0];
        let out_width = output_size[1];
        let img = self.clone();
        if out_height <= 0 || out_width <= 0 {
            return Err(ShapeError::ConvError {
                message: if out_height <= 0 {
                    "output height <= 0".to_string()
                } else {
                    "output width <= 0".to_string()
                },
                location: core::panic::Location::caller(),
            }
            .into());
        }
        let output = _Tensor::<T>::empty([batch, out_height, out_width, in_channels])?;
        let out = output.ptr();
        let inp = img.ptr();

        let osb = output.strides()[0]; // batch
        let osh = output.strides()[1]; // height
        let osw = output.strides()[2]; // width

        let isb = img.strides()[0]; // batch
        let ish = img.strides()[1]; // height
        let isw = img.strides()[2]; // width

        let out_size = batch * out_height * out_width;

        const IC_BLOCK_SIZE: usize = REGNUM / 2;
        let in_channel_remain = in_channels % ((IC_BLOCK_SIZE * T::Vec::SIZE) as i64);
        (0..out_size).into_par_iter().for_each(|idx| {
            let out = out.clone();
            let b = idx / (out_height * out_width);
            let h = (idx / out_width) % out_height;
            let w = idx % out_width;
            let start_h = (h * img_height / out_height) as i64;
            let end_h = ((h + 1) * img_height + out_height - 1) / out_height as i64;
            let start_w = (w * img_width / out_width) as i64;
            let end_w = ((w + 1) * img_width + out_width - 1) / out_width as i64;
            let kernel_size: T = ((end_h - start_h) * (end_w - start_w)).into_scalar();
            let kernel_size_vec = T::Vec::splat(kernel_size);
            for ii in (0..in_channels - in_channel_remain).step_by(IC_BLOCK_SIZE * T::Vec::SIZE) {
                let mut res_vecs = [T::Vec::splat(T::ZERO); IC_BLOCK_SIZE];
                for kh in start_h..end_h {
                    for kw in start_w..end_w {
                        let mut inp_vecs = [T::Vec::splat(T::ZERO); IC_BLOCK_SIZE];
                        for (idx, vec) in inp_vecs.iter_mut().enumerate() {
                            let i = ii + ((idx * T::Vec::SIZE) as i64);
                            let inp_idx = b * isb + kh * ish + kw * isw + i;
                            *vec = unsafe { T::Vec::from_ptr(&inp[inp_idx]) };
                        }

                        avgpool2d_kernel::<T, IC_BLOCK_SIZE>(&inp_vecs, &mut res_vecs);
                    }
                }
                for (idx, vec) in res_vecs.iter().enumerate() {
                    let i = ii + ((idx * T::Vec::SIZE) as i64);
                    let out_idx = b * osb + h * osh + w * osw + i;
                    let out_vec = (unsafe { out.ptr.add(out_idx as usize) }) as *mut T::Vec;
                    unsafe {
                        out_vec.write_unaligned(vec.read_unaligned()._div(kernel_size_vec));
                    }
                }
            }

            let remain = in_channel_remain % (T::Vec::SIZE as i64);
            for ii in (in_channels - in_channel_remain..in_channels - remain).step_by(T::Vec::SIZE)
            {
                let mut res_vecs = T::Vec::splat(T::ZERO);
                for kh in start_h..end_h {
                    for kw in start_w..end_w {
                        let i = ii;
                        let inp_idx = b * isb + kh * ish + kw * isw + i;
                        let inp_vec = unsafe { T::Vec::from_ptr(&inp[inp_idx]) };

                        res_vecs = res_vecs._add(inp_vec);
                    }
                }
                let i = ii;
                let out_idx = b * osb + h * osh + w * osw + i;
                let out_vec = (unsafe { out.ptr.add(out_idx as usize) }) as *mut T::Vec;
                unsafe {
                    out_vec.write_unaligned(res_vecs.read_unaligned()._div(kernel_size_vec));
                }
            }

            for ii in in_channels - remain..in_channels {
                let mut res = T::ZERO;
                for kh in start_h..end_h {
                    for kw in start_w..end_w {
                        let i = ii;
                        let inp_idx = b * isb + kh * ish + kw * isw + i;

                        res = res._add(inp[inp_idx]);
                    }
                }
                let i = ii;
                let out_idx = b * osb + h * osh + w * osw + i;
                let out = (unsafe { out.ptr.add(out_idx as usize) }) as *mut T;
                unsafe {
                    out.write_unaligned(res._div(kernel_size));
                }
            }
        });

        Ok(output)
    }
}

fn avgpool2d_kernel<T: CommonBounds, const IC_BLOCK_SIZE: usize>(
    inps: &[T::Vec; IC_BLOCK_SIZE],
    outs: &mut [T::Vec; IC_BLOCK_SIZE],
) {
    for idx in 0..IC_BLOCK_SIZE {
        outs[idx] = outs[idx]._add(inps[idx]);
    }
}

impl<T> Tensor<T>
where
    T: CommonBounds + IntoScalar<T> + NormalOut<Output = T> + FloatOutBinary<T, Output = T>,
    T::Vec: VecTrait<T>
        + Copy
        + Send
        + Sync
        + NormalOut<Output = T::Vec>
        + FloatOutBinary<T::Vec, Output = T::Vec>,
    bool: IntoScalar<T>,
    i64: IntoScalar<T>,
{
    /// Performs a 2D avg pooling operation on the input tensor.
    ///
    /// This method applies a 2D avg pooling operation on the tensor using the specified kernel,
    /// strides (steps), padding, and dilation factors.
    ///
    /// # Arguments
    ///
    /// * `kernels` - A reference to the tensor representing the convolution kernels (filters).
    ///   The size of the kernel tensor determines the spatial dimensions of the convolution operation.
    /// * `steps` - A 2-element array specifying the stride (step size) of the convolution along the height and width dimensions.
    /// * `padding` - A 2-element array of tuples representing the padding for the height and width dimensions.
    ///   Each tuple specifies the amount of padding added before and after the data along the respective axis.
    /// * `dilation` - A 2-element array specifying the dilation factor for the convolution along the height and width dimensions.
    ///   Dilation allows the kernel to be applied to inputs with gaps, increasing the receptive field of the kernel.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing the output tensor after applying the 2D avg pooling operation.
    #[cfg_attr(feature = "track_caller", track_caller)]
    #[inline(never)]
    pub fn adaptive_avgpool2d(
        &self,
        output_size: [i64; 2],
    ) -> std::result::Result<Tensor<T>, TensorError> {
        Ok(self.inner.adaptive_avgpool2d(output_size)?.into())
    }
}
