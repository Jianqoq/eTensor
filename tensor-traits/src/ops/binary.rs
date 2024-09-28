use std::borrow::{ Borrow, BorrowMut };

use anyhow::Result;
use tensor_types::dtype::TypeCommon;

use crate::tensor::CommonBounds;

/// A trait for binary operations on tensors.
pub trait NormalBinOps<RHS = Self>
    where <<Self as NormalBinOps<RHS>>::OutputMeta as TypeCommon>::Vec: Send + Sync
{
    /// The output tensor type.
    type Output;
    /// The output tensor data type.
    type OutputMeta: CommonBounds;
    /// The inplace output tensor type.
    type InplaceOutput;

    /// inplace version of add
    ///
    /// # See Also
    ///
    /// - [`add`]: Perform addition of `self` and `rhs` element-wise, with auto broadcasting.
    fn add_<U>(&self, rhs: RHS, out: U) -> Result<Self::Output>
        where U: Borrow<Self::InplaceOutput>;

    /// Inplace version of subtraction
    ///
    /// # See Also
    ///
    /// - [`sub`]: Perform subtraction of `self` and `rhs` element-wise, with auto broadcasting.
    fn sub_<U>(&self, rhs: RHS, out: U) -> Result<Self::Output>
        where U: Borrow<Self::InplaceOutput>;

    /// Inplace version of multiplication
    ///
    /// # See Also
    ///
    /// - [`mul`]: Perform multiplication of `self` and `rhs` element-wise, with auto broadcasting.
    fn mul_<U>(&self, rhs: RHS, out: U) -> Result<Self::Output>
        where U: Borrow<Self::InplaceOutput>;

    /// Inplace version of rem
    ///
    /// # See Also
    ///
    /// - [`div`]: Perform rem of `self` and `rhs` element-wise, with auto broadcasting.
    fn rem_<U>(&self, rhs: RHS, out: U) -> Result<Self::Output>
        where U: Borrow<Self::InplaceOutput>;
}

/// A trait for matrix multiplication operations on tensors.
pub trait Matmul<RHS = Self>
    where <<Self as Matmul<RHS>>::OutputMeta as TypeCommon>::Vec: Send + Sync
{
    /// The output tensor type.
    type Output;
    /// The output tensor data type.
    type OutputMeta: CommonBounds;
    /// The inplace output tensor type.
    type InplaceOutput;

    /// Computes the matrix multiplication of two tensors.
    ///
    /// The `matmul` function performs matrix multiplication on two input tensors. This operation supports both 2D matrices
    /// and higher-dimensional tensors. For higher-dimensional tensors, this function treats the last two dimensions as matrices
    /// and performs matrix multiplication over them, broadcasting the remaining dimensions.
    ///
    /// # Parameters
    ///
    /// - `A`: The first tensor to be multiplied.
    /// - `B`: The second tensor to be multiplied.
    ///
    /// # Returns
    ///
    /// - `anyhow::Result<_Tensor<FloatType<T>>>`: A tensor containing the result of the matrix multiplication.
    ///
    /// # Notes
    ///
    /// - **Matrix Multiplication**: Performs matrix multiplication between two tensors. The number of columns in the first matrix
    ///   must match the number of rows in the second matrix.
    /// - **Broadcasting**: For higher-dimensional tensors, the function broadcasts over the batch dimensions and performs matrix
    ///   multiplication on the last two dimensions.
    /// - **Compatibility**: The input tensors must have compatible shapes for matrix multiplication.
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn matmul(&self, rhs: RHS) -> Result<Self::Output>;

    /// Inplace version of matmul
    ///
    /// # See Also
    ///
    /// - [`matmul`]: Perform matrix multiplication of `self` and `rhs`.
    #[cfg_attr(feature = "track_caller", track_caller)]
    fn matmul_<U>(&self, rhs: RHS, out: U) -> Result<Self::Output>
        where U: Borrow<Self::InplaceOutput> + BorrowMut<Self::InplaceOutput>;
}
