use std::{ panic::Location, sync::{ Arc, Barrier } };

use tensor_common::{
    err_handler::ErrHandler,
    prg_update::next_sub1,
    shape_utils::mt_intervals,
    slice::Slice,
};
use tensor_traits::{
    shape_manipulate::ShapeManipulate,
    tensor::{ CommonBounds, TensorCreator, TensorInfo },
};

use crate::{ tensor_base::_Tensor, THREAD_POOL };

/// Concatenates multiple tensors along a specified axis.
///
/// This method concatenates a list of tensors along a specified axis, with an option to retain
/// or collapse dimensions. All tensors must have the same shape except for the concatenation axis.
///
/// # Arguments
///
/// * `tensors` - A vector of references to the tensors that will be concatenated.
/// * `axis` - The axis along which the concatenation will occur. All tensors must have
///   the same shape along the non-concatenation axes.
/// * `keepdims` - A boolean flag indicating whether to retain the original dimensions of
///   the tensors in the output:
///   - If `true`, the original dimensions will be kept.
///   - If `false`, the resulting tensor will have its dimensions adjusted based on concatenation.
///
/// # Returns
///
/// This function returns a `Result` containing a new tensor that is the result of concatenating
/// the input tensors along the specified axis.
#[cfg_attr(feature = "track_caller", track_caller)]
pub(crate) fn concat<T>(
    tensors: Vec<&_Tensor<T>>,
    axis: usize,
    keepdims: bool
) -> anyhow::Result<_Tensor<T>>
    where T: CommonBounds
{
    let length = tensors.len();
    let mut all_same_shape = true;
    for i in tensors.iter() {
        tensors[0]
            .shape()
            .iter()
            .enumerate()
            .try_for_each(|(idx, x)| {
                if
                    idx != axis &&
                    i.shape().len() == tensors[0].shape().len() &&
                    *x != i.shape()[idx]
                {
                    return Err(
                        anyhow::Error::msg("Shapes except the axis to stack must be the same")
                    );
                } else if i.shape().len() != tensors[0].shape().len() {
                    return Err(
                        ErrHandler::NdimMismatched(
                            tensors[0].ndim(),
                            i.ndim(),
                            Location::caller()
                        ).into()
                    );
                } else if idx == axis && *x != i.shape()[idx] {
                    all_same_shape = false;
                }
                Ok(())
            })?;
    }
    let mut new_shape: Vec<i64> = vec![0; tensors[0].ndim()];
    tensors.iter().for_each(|x| {
        new_shape[axis] += x.shape()[axis];
    });
    tensors[0]
        .shape()
        .iter()
        .enumerate()
        .for_each(|(i, x)| {
            if i != axis {
                new_shape[i] = *x;
            }
        });
    let new_tensor = _Tensor::<T>::empty(&new_shape)?;
    let mut begin = 0;
    let mut res_slices = Vec::with_capacity(length);
    for i in tensors.iter() {
        let mut selections = vec![Slice::Full; new_shape.len()];
        selections[axis] = Slice::Range((begin, begin + i.shape()[axis]));
        begin += i.shape()[axis];
        let res_tensor = new_tensor.slice(&selections)?;
        res_slices.push(res_tensor);
    }
    let tensors = tensors
        .iter()
        .map(|x| (*x).clone())
        .collect::<Vec<_Tensor<T>>>();
    THREAD_POOL.with_borrow_mut(|pool| {
        let num_threads: usize;
        if length < pool.max_count() {
            num_threads = length;
        } else {
            num_threads = pool.max_count();
        }
        let barrier = Arc::new(Barrier::new(num_threads + 1));
        let intervals: Vec<(usize, usize)> = mt_intervals(length, num_threads);
        for i in 0..num_threads {
            let barrier_clone = Arc::clone(&barrier);
            let (start, end) = intervals[i];
            let res_tensors = res_slices[start..end].to_vec();
            let inputs = tensors[start..end].to_vec();
            pool.execute(move || {
                for (res, input) in res_tensors.into_iter().zip(inputs.into_iter()) {
                    let mut res_ptr = res.ptr();
                    let mut a_data = input.ptr();
                    let a_last_stride = *input.strides().last().unwrap();
                    let inner_loop_size = *input.shape().last().unwrap();
                    let outer_loop_size = input.size() / (inner_loop_size as usize);
                    let mut prg = vec![0; input.ndim()];
                    for _ in 0..outer_loop_size {
                        for i in 0..inner_loop_size {
                            let a_val = a_data[i * a_last_stride];
                            res_ptr[i] = a_val;
                        }
                        next_sub1(
                            &mut prg,
                            input.shape(),
                            [&mut a_data, &mut res_ptr],
                            [&input.shape(), &res.shape()],
                            [&input.strides(), &res.strides()]
                        );
                    }
                }
                barrier_clone.wait();
            });
        }
        barrier.wait();
    });
    if keepdims {
        let mut res_shape = Vec::with_capacity(new_shape.len() + 1);
        for (idx, i) in new_shape.iter().enumerate() {
            if idx == axis {
                res_shape.push(length as i64);
                res_shape.push(*i / (length as i64));
            } else {
                res_shape.push(*i);
            }
        }
        new_tensor.reshape(res_shape)
    } else {
        Ok(new_tensor)
    }
}
