use crate::backend::Cpu;
use crate::tensor_base::_Tensor;
use crate::THREAD_POOL;
use std::sync::{ Arc, Barrier };
use tensor_common::shape_utils::mt_intervals;
use tensor_traits::{ CommonBounds, TensorCreator, TensorInfo };
use tensor_types::type_promote::NormalOut;

impl<T> _Tensor<T, Cpu> where T: CommonBounds + NormalOut<Output = T> {
    pub fn conv(
        &self,
        kernel: &_Tensor<T, Cpu>,
        bias: Option<&_Tensor<T, Cpu>>,
        kernel_shape: Option<&[i64]>,
        steps: Option<&[i64]>,
        pads: Option<&[(i64, i64)]>,
        dilation: Option<&[i64]>,
        out_pads: Option<&[(i64, i64)]>,
        group: Option<i64>
    ) -> anyhow::Result<Self> {
        let _kernel_shape = if let Some(_kernel_shape) = kernel_shape {
            _kernel_shape.to_vec()
        } else {
            kernel.shape().iter().skip(2).cloned().collect::<Vec<_>>()
        };
        let process_pads = |pads: Option<&[(i64, i64)]>| {
            if let Some(_pads) = pads {
                _pads
                    .iter()
                    .map(|(a, b)| (*a, *b))
                    .collect::<Vec<_>>()
            } else {
                vec![(0, 0); _kernel_shape.len()]
            }
        };
        let _pads = process_pads(pads);
        let _out_pads = process_pads(out_pads);
        let _steps = if let Some(_steps) = steps {
            _steps.to_vec()
        } else {
            vec![1; _kernel_shape.len()]
        };
        let _dilation = if let Some(_dilation) = dilation {
            Arc::new(
                _dilation
                    .iter()
                    .map(|a| *a)
                    .collect::<Vec<_>>()
            )
        } else {
            Arc::new(vec![1; _kernel_shape.len()])
        };
        let _groups = group.unwrap_or(1);
        let mut outer_dims = vec![];
        outer_dims.push(self.shape()[0]);
        outer_dims.push(kernel.shape()[0]);
        let mut out_dims = vec![];
        self.shape()
            .iter()
            .skip(2)
            .zip(kernel.shape().iter().skip(2))
            .enumerate()
            .for_each(|(idx, (x, y))| {
                let i = *x;
                let (p_begin, p_end) = _pads[idx];
                let d = _dilation[idx];
                let s = _steps[idx];
                let k = *y;
                let o = (i + p_begin + p_end - d * (k - 1) - 1) / s + 1;
                let (p_begin, p_end) = _out_pads[idx];
                out_dims.push(o + p_begin + p_end);
            });
        let kernel_per_group = kernel.shape()[0] / _groups;
        let in_channels_per_group = self.shape()[1];
        let mut loop_shape = vec![1; 4 + _kernel_shape.len()];
        loop_shape[0] = self.shape()[0];
        loop_shape[1] = _groups;
        loop_shape[2] = kernel_per_group;
        loop_shape[3] = in_channels_per_group;
        loop_shape[4..].copy_from_slice(&out_dims);
        let loop_shape = Arc::new(loop_shape);
        outer_dims.append(&mut out_dims);
        let ret = _Tensor::<T, Cpu>::zeros(outer_dims)?;

        let outer_loop_size = loop_shape.iter().product::<i64>() / loop_shape.last().unwrap();
        let inner_loop_size = *loop_shape.last().unwrap();
        let kernel_outer_loop_size =
            _kernel_shape.iter().product::<i64>() / _kernel_shape.last().unwrap();
        let kernel_inner_loop_size = *_kernel_shape.last().unwrap();
        let kernel_ndim = _kernel_shape.len();
        let kernal_shape = Arc::new(_kernel_shape);
        THREAD_POOL.with_borrow_mut(|pool| {
            let num_threads = if (outer_loop_size as usize) < pool.max_count() {
                outer_loop_size as usize
            } else {
                pool.max_count()
            };
            let intervals = mt_intervals(outer_loop_size as usize, num_threads);
            let mut prgs = vec![];
            let mut res_ptrs = vec![];
            let mut ptrs = vec![];
            let mut kernel_ptrs = vec![];
            let mut inp_prg_mask = vec![0; loop_shape.len()];
            let mut kernel_prg_mask = vec![0; loop_shape.len()];
            let mut res_prg_mask = vec![0; loop_shape.len()];
            for (idx, j) in (0..loop_shape.len()).enumerate().rev() {
                if
                    j == 0
                    /* batch */
                {
                    inp_prg_mask[j] = self.strides()[0];
                    res_prg_mask[j] = ret.strides()[0];
                } else if
                    j == 1
                    /* _group idx */
                {
                    res_prg_mask[j] = kernel_per_group * ret.strides()[1];
                    inp_prg_mask[j] = in_channels_per_group * self.strides()[1];
                    kernel_prg_mask[j] = kernel_per_group * kernel.strides()[0];
                } else if j == 2 {
                    res_prg_mask[j] = ret.strides()[1];
                    kernel_prg_mask[j] = kernel.strides()[0];
                } else if j == 3 {
                    inp_prg_mask[j] = self.strides()[1];
                    kernel_prg_mask[j] = kernel.strides()[1];
                } else {
                    res_prg_mask[j] = ret.strides()[idx - 2];
                    inp_prg_mask[j] = _steps[idx - 4] * self.strides()[idx - 4 + 2];
                }
            }
            let inp_prg_mask = Arc::new(inp_prg_mask);
            let kernel_prg_mask = Arc::new(kernel_prg_mask);
            let res_prg_mask = Arc::new(res_prg_mask);

            for (start, _) in intervals.iter() {
                let mut amount = *start * (inner_loop_size as usize);
                let mut current_prg: Vec<i64> = vec![0; loop_shape.len()];
                let mut res_offset = 0;
                let mut inp_offset = 0;
                let mut kernel_offset = 0;
                // inp[batch * inp.strides[0] + g * in_channel * inp.strides[1] + c * inp.strides[1] + ...]
                for j in (0..loop_shape.len()).rev() {
                    current_prg[j] = (amount as i64) % loop_shape[j];
                    amount /= loop_shape[j] as usize;
                    inp_offset += current_prg[j] * inp_prg_mask[j];
                    res_offset += current_prg[j] * res_prg_mask[j];
                    kernel_offset += current_prg[j] * kernel_prg_mask[j];
                }
                let mut inp_ptr = self.ptr();
                let mut res_ptr = ret.ptr();
                let mut kernel_ptr = kernel.ptr();
                inp_ptr.offset(inp_offset);
                res_ptr.offset(res_offset);
                kernel_ptr.offset(kernel_offset);
                prgs.push(current_prg);
                ptrs.push(inp_ptr);
                kernel_ptrs.push(kernel_ptr);
                res_ptrs.push(res_ptr);
            }
            let barrier = Arc::new(Barrier::new(num_threads + 1));
            for (((((start, end), mut prg), mut inp_ptr), mut res_ptr), mut kernel_ptr) in intervals
                .into_iter()
                .zip(prgs.into_iter())
                .zip(ptrs.into_iter())
                .zip(res_ptrs.into_iter())
                .zip(kernel_ptrs.into_iter()) {
                let barrier_clone = barrier.clone();
                let mut reduce_prg = vec![0; kernel_ndim];
                let inp_strides = self.strides().clone();
                let _kernel_shape = kernal_shape.clone();
                let dilations = _dilation.clone();
                let inp_last_strides = *inp_strides.last().unwrap();
                let last_dilation = *dilations.last().unwrap();
                let kernel_last_strides = *kernel.strides().last().unwrap();
                let ret_last_stride = *ret.strides().last().unwrap();
                let loop_shape = loop_shape.clone();
                let kernel_strides = kernel.strides().clone();
                let _steps = _steps.to_vec();
                let last_steps = *_steps.last().unwrap();
                let inp_prg_mask = inp_prg_mask.clone();
                let kernel_prg_mask = kernel_prg_mask.clone();
                let res_prg_mask = res_prg_mask.clone();
                pool.execute(move || {
                    let inp_reduce_strides = &inp_strides[2..];
                    let _kernel_strides = &kernel_strides[2..];
                    for _ in start..end {
                        for x in 0..inner_loop_size {
                            let mut sum = T::ZERO;
                            for _ in 0..kernel_outer_loop_size {
                                for i in 0..kernel_inner_loop_size {
                                    let val =
                                        inp_ptr
                                            [x * last_steps + i * inp_last_strides * last_dilation];
                                    let kernel_val = kernel_ptr[i * kernel_last_strides];
                                    sum = sum._add(val._mul(kernel_val));
                                }
                                for j in (0..kernel_ndim - 1).rev() {
                                    if reduce_prg[j] < _kernel_shape[j] - 1 {
                                        reduce_prg[j] += 1;
                                        inp_ptr.offset(dilations[j] * inp_reduce_strides[j]);
                                        kernel_ptr.offset(_kernel_strides[j]);
                                        break;
                                    } else {
                                        reduce_prg[j] = 0;
                                        inp_ptr.offset(
                                            -dilations[j] *
                                                inp_reduce_strides[j] *
                                                (_kernel_shape[j] - 1)
                                        );
                                        kernel_ptr.offset(
                                            -_kernel_strides[j] * (_kernel_shape[j] - 1)
                                        );
                                    }
                                }
                            }
                            let res_val = res_ptr[x * ret_last_stride];
                            res_ptr.modify(x * ret_last_stride, res_val._add(sum));
                        }
                        for k in (0..loop_shape.len() - 1).rev() {
                            if prg[k] < loop_shape[k] - 1 {
                                prg[k] += 1;
                                inp_ptr.offset(inp_prg_mask[k]);
                                res_ptr.offset(res_prg_mask[k]);
                                kernel_ptr.offset(kernel_prg_mask[k]);
                                break;
                            } else {
                                prg[k] = 0;
                                inp_ptr.offset(-inp_prg_mask[k] * (loop_shape[k] - 1));
                                res_ptr.offset(-res_prg_mask[k] * (loop_shape[k] - 1));
                                kernel_ptr.offset(-kernel_prg_mask[k] * (loop_shape[k] - 1));
                            }
                        }
                    }
                    barrier_clone.wait();
                });
            }
            barrier.wait();
        });
        Ok(ret)
    }
}
