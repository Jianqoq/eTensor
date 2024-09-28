use super::conv_config::Conv2dConfig;
use crate::ops::cpu::conv_config::KernelParamAlgo;
use crate::tensor_base::_Tensor;
use crate::CONV_REGNUM;
use rayon::prelude::*;
use tensor_common::err_handler::ErrHandler;
use tensor_common::err_handler::ErrHandler::InvalidInputShape;
use tensor_traits::CommonBounds;
use tensor_traits::TensorCreator;
use tensor_traits::TensorInfo;
use tensor_types::into_scalar::IntoScalar;
use tensor_types::type_promote::NormalOut;
use tensor_types::vectors::traits::*;

impl<T> _Tensor<T>
    where
        T: CommonBounds + IntoScalar<T> + NormalOut<Output = T>,
        T::Vec: VecTrait<T> + Copy + Init<T> + Send + Sync + VecCommon + NormalOut<Output = T::Vec>
{
    /// Performs a 2D convolution operation on the input tensor.
    ///
    /// This method applies a 2D convolution operation on the tensor using the specified kernel,
    /// strides (steps), padding, and dilation factors. It optionally accepts a configuration (`Conv2dConfig`)
    /// to fine-tune the performance, such as optimizing for cache usage and block sizes.
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
    /// * `config` - An optional reference to a `Conv2dConfig` structure that holds additional configuration parameters for optimization.
    ///   If not provided, a default configuration is used.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing the output tensor after applying the 2D convolution operation.
    #[cfg_attr(feature = "track_caller", track_caller)]
    pub fn iconv2d(
        &self,
        kernels: &_Tensor<T>,
        steps: [i64; 2],
        padding: [(i64, i64); 2],
        dilation: [i64; 2],
        config: Option<&Conv2dConfig<T>>
    ) -> anyhow::Result<_Tensor<T>> {
        let img_shape = self.shape();
        if img_shape.len() != 4 {
            return Err(
                ErrHandler::Conv2dImgShapeInCorrect(
                    img_shape.len(),
                    core::panic::Location::caller()
                ).into()
            );
        }
        let batch = img_shape[0];
        let img_height = img_shape[1];
        let img_width = img_shape[2];
        let img_channels = img_shape[3];
        let kernel_shape = kernels.shape();
        let out_channels = kernel_shape[0];
        let kernel_height = kernel_shape[1];
        let kernel_width = kernel_shape[2];
        let in_channels = kernel_shape[3];
        if in_channels != img_channels {
            panic!(
                "The number of input channels in the image must be equal to the number of input channels in the kernel."
            );
        }
        let (step_width, step_height) = (steps[0], steps[1]);
        let ((ph_start, ph_end), (pw_start, pw_end)) = (padding[0], padding[1]);
        let (dh, dw) = (dilation[0], dilation[1]);

        let out_height =
            (img_height + ph_start + ph_end - dh * (kernel_height - 1) - 1) / step_height + 1;
        let out_width =
            (img_width + pw_start + pw_end - dw * (kernel_width - 1) - 1) / step_width + 1;
        let img = if !padding.iter().all(|(a, b)| *a == 0 && *b == 0) {
            self.pad(
                &[
                    (0, 0),
                    (ph_start, ph_end),
                    (pw_start, pw_end),
                    (0, 0),
                ],
                T::ZERO
            )?
        } else {
            self.clone()
        };
        if out_height <= 0 || out_width <= 0 {
            return if out_height <= 0 {
                Err(InvalidInputShape(out_height, core::panic::Location::caller()).into())
            } else {
                Err(InvalidInputShape(out_width, core::panic::Location::caller()).into())
            };
        }
        let output = _Tensor::<T>::empty([batch, out_height, out_width, out_channels])?;
        let mut out = output.ptr();
        let inp = img.ptr();
        let kernel = kernels.ptr();

        let osb = output.strides()[0]; // batch
        let osh = output.strides()[1]; // height
        let osw = output.strides()[2]; // width

        let isb = img.strides()[0]; // batch
        let ish = img.strides()[1]; // height
        let isw = img.strides()[2]; // width

        let ks0 = kernels.strides()[0]; // kernel_out_channels
        let ks1 = kernels.strides()[1]; // kernel_height
        let ks2 = kernels.strides()[2]; // kernel_width

        let num_oh = out_height / 3;
        let outer = batch * num_oh;
        (0..outer).into_par_iter().for_each(|idx| {
            let mut out = output.ptr();
            let b = idx / num_oh;
            let lp = idx % num_oh;
            for k in (0..out_width).step_by(4) {
                for j in 0..out_channels {
                    for i in (0..in_channels).step_by(T::Vec::SIZE * 2) {
                        for l in 0..3 {
                            let l = lp * 3 + l;
                            let mut out_regs = [T::Vec::splat(T::ZERO); 4];
                            for n in 0..kernel_height {
                                let inp_offset = b * isb + (l * step_height + n * dh) * ish;
                                for m in 0..kernel_width {
                                    let koffset = j * ks0 + n * ks1 + m * ks2;
                                    unsafe {
                                        let kernel_vec = T::Vec::from_ptr(&kernel[koffset + i]); // prettier-ignore
                                        let inp0 = T::Vec::from_ptr(&inp[inp_offset + (k * step_width + m * dw) * isw + i]); // prettier-ignore
                                        let inp1 = T::Vec::from_ptr(&inp[inp_offset + ((k + 1) * step_width + m * dw) * isw + i]); // prettier-ignore
                                        let inp2 = T::Vec::from_ptr(&inp[inp_offset + ((k + 2) * step_width + m * dw) * isw + i]); // prettier-ignore
                                        let inp3 = T::Vec::from_ptr(&inp[inp_offset + ((k + 3) * step_width + m * dw) * isw + i]); // prettier-ignore
                                        out_regs[0] = kernel_vec.mul_add(inp0, out_regs[0]);
                                        out_regs[1] = kernel_vec.mul_add(inp1, out_regs[1]);
                                        out_regs[2] = kernel_vec.mul_add(inp2, out_regs[2]);
                                        out_regs[3] = kernel_vec.mul_add(inp3, out_regs[3]);
                                        let kernel_vec = T::Vec::from_ptr(&kernel[koffset + i + T::Vec::SIZE as i64]); // prettier-ignore
                                        let inp0 = T::Vec::from_ptr(&inp[inp_offset + (k * step_width + m * dw) * isw + i + T::Vec::SIZE as i64]); // prettier-ignore
                                        let inp1 = T::Vec::from_ptr(&inp[inp_offset + ((k + 1) * step_width + m * dw) * isw + i + T::Vec::SIZE as i64]); // prettier-ignore
                                        let inp2 = T::Vec::from_ptr(&inp[inp_offset + ((k + 2) * step_width + m * dw) * isw + i + T::Vec::SIZE as i64]); // prettier-ignore
                                        let inp3 = T::Vec::from_ptr(&inp[inp_offset + ((k + 3) * step_width + m * dw) * isw + i + T::Vec::SIZE as i64]); // prettier-ignore
                                        out_regs[0] = kernel_vec.mul_add(inp0, out_regs[0]);
                                        out_regs[1] = kernel_vec.mul_add(inp1, out_regs[1]);
                                        out_regs[2] = kernel_vec.mul_add(inp2, out_regs[2]);
                                        out_regs[3] = kernel_vec.mul_add(inp3, out_regs[3]);
                                    }
                                }
                            }
                            out[b * osb + l * osh + k * osw + j] = out_regs[0].sum();
                            out[b * osb + l * osh + (k + 1) * osw + j] = out_regs[1].sum();
                            out[b * osb + l * osh + (k + 2) * osw + j] = out_regs[2].sum();
                            out[b * osb + l * osh + (k + 3) * osw + j] = out_regs[3].sum();
                        }
                    }
                }
            }
        });

        Ok(output)
    }
}
