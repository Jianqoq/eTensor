use std::panic::Location;

use thiserror::Error;

use crate::{ shape::Shape, strides::Strides };
#[derive(Debug, Error)]
pub enum ErrHandler {
    #[error("expect size {0} but got size {1}")] SizeMismatched(i64, i64),
    #[error(
        "lhs matrix shape is {0:?}, rhs matrix shape is {1:?}, expect rhs matrix shape to be [{2}, any]"
    )] MatmulShapeMismatched([i64; 2], [i64; 2], i64),
    #[error("expect {0} but got {1}")] NdimMismatched(usize, usize),
    #[error("tensor ndim is {0} but got index {1} => {2}")] IndexOutOfRange(usize, i64, i64),
    #[error("Shape mismatched: {0}")] IndexRepeated(String),
    #[error("Shape mismatched: {0}")] ExpandDimError(String),
    #[error(
        "can't perform inplace reshape to from {0} to {1} with strides {2}"
    )] IterInplaceReshapeError(Shape, Shape, Strides),
    #[error("can't broacast lhs: {0} with rhs: {1}, expect lhs_shape[{2}] to be 1")] BroadcastError(
        Shape,
        Shape,
        usize,
        &'static Location<'static>,
    ),
    #[error("axis should be unique, but got {0} and {1}")] SameAxisError(i64, i64),
    #[error("can't reshape from {0} with size {2} to {1} with size {3}, at {4}")] ReshapeError(
        Shape,
        Shape,
        usize,
        usize,
        &'static Location<'static>,
    ),
}

impl ErrHandler {
    pub fn check_ndim_match(ndim: usize, expect_ndim: usize) -> Result<(), Self> {
        if ndim != expect_ndim {
            return Err(ErrHandler::NdimMismatched(expect_ndim, ndim));
        }
        Ok(())
    }
    pub fn check_same_axis(axis1: i64, axis2: i64) -> Result<(), Self> {
        if axis1 == axis2 {
            return Err(ErrHandler::SameAxisError(axis1, axis2));
        }
        Ok(())
    }
    pub fn check_index_in_range(ndim: usize, index: &mut i64) -> Result<(), Self> {
        let indedx = if *index < 0 { *index + (ndim as i64) } else { *index };
        if indedx < 0 || indedx >= (ndim as i64) {
            return Err(ErrHandler::IndexOutOfRange(ndim, *index, indedx));
        }
        *index = indedx;
        Ok(())
    }
    pub fn check_size_match(size1: i64, size2: i64) -> Result<(), Self> {
        if size1 != size2 {
            return Err(ErrHandler::SizeMismatched(size1, size2));
        }
        Ok(())
    }
}
