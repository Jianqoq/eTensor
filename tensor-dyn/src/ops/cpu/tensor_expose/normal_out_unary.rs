use std::borrow::Borrow;

use crate::{
    ops::cpu::tensor_internal::normal_out_unary::NormalType,
    tensor::Tensor,
    tensor_base::_Tensor,
};
use anyhow::Result;
use tensor_traits::{ CommonBounds, NormalUaryOps, TensorLike };
use tensor_types::{ into_scalar::IntoScalar, type_promote::NormalOutUnary };

impl<T> NormalUaryOps
    for Tensor<T>
    where
        T: CommonBounds + IntoScalar<T>,
        NormalType<T>: CommonBounds,
        T::Vec: NormalOutUnary<Base = NormalType<T>>,
        T: NormalOutUnary<Base = NormalType<T>>,
        _Tensor<NormalType<T>>: TensorLike<NormalType<T>>
{
    type Output = Tensor<NormalType<T>>;

    type InplaceOutput = _Tensor<NormalType<T>>;

    type OutputMeta = NormalType<T>;

    /// Computes the element-wise floor of the tensor.
    ///
    /// This function rounds each element in the tensor down to the nearest integer, returning a new tensor
    /// where each element is the largest integer less than or equal to the corresponding element in the original tensor.
    /// # Arguments
    /// This function takes no arguments.
    /// # Returns
    /// * A new tensor where each element is the floor of the corresponding element in the original tensor.
    /// # Panics
    /// * This function should not panic under normal conditions.
    /// # Examples
    /// ```
    /// use tensor_dyn::tensor::Tensor;
    /// use tensor_dyn::NormalUaryOps;
    /// let a = Tensor::<f64>::new([0.1, 1.5, 2.9, 3.0]);
    /// let b = a.floor().unwrap();
    /// ```
    fn floor(&self) -> Result<Self::Output> {
        Ok(_Tensor::floor(self)?.into())
    }

    fn floor_<U>(&self, out: U) -> Result<Self::Output> where U: Borrow<Self::InplaceOutput> {
        Ok(_Tensor::floor_(self, out)?.into())
    }

    /// Computes the element-wise square of the tensor.
    ///
    /// This function returns a new tensor where each element is the square of the corresponding element in the original tensor:
    ///
    /// `square(x) = x^2`
    /// # Arguments
    /// This function takes no arguments.
    /// # Returns
    /// * A new tensor where each element is squared.
    /// # Panics
    /// * This function should not panic under normal conditions.
    /// # Examples
    /// ```
    /// use tensor_dyn::tensor::Tensor;
    /// use tensor_dyn::NormalUaryOps;
    /// let a = Tensor::<f64>::new([0.1, 1.5, 2.9, 3.0]);
    /// let b = a.square().unwrap();
    /// ```
    fn square(&self) -> Result<Self::Output> {
        Ok(_Tensor::square(self)?.into())
    }

    fn square_<U>(&self, out: U) -> Result<Self::Output> where U: Borrow<Self::InplaceOutput> {
        Ok(_Tensor::square_(self, out)?.into())
    }

    /// Computes the element-wise absolute value of the tensor.
    ///
    /// This function returns a new tensor where each element is the absolute value of the corresponding element in the original tensor:
    ///
    /// `abs(x) = |x|`
    ///
    /// # Arguments
    ///
    /// This function takes no arguments.
    ///
    /// # Returns
    ///
    /// * A new tensor where each element is the absolute value of the corresponding element in the original tensor.
    ///
    /// # Panics
    ///
    /// * This function should not panic under normal conditions.
    /// # Examples
    /// ```
    /// use tensor_dyn::tensor::Tensor;
    /// use tensor_dyn::NormalUaryOps;
    /// let a = Tensor::<f64>::new([-1.0, 1.5, -2.9, 3.0]);
    /// let b = a.abs().unwrap();
    /// ```
    fn abs(&self) -> Result<Self> {
        Ok(_Tensor::abs(self)?.into())
    }

    fn abs_<U>(&self, out: U) -> Result<Self> where U: Borrow<Self::InplaceOutput> {
        Ok(_Tensor::abs_(self, out)?.into())
    }

    /// Computes the element-wise ceiling of the tensor.
    ///
    /// This function rounds each element in the tensor up to the nearest integer, returning a new tensor
    /// where each element is the smallest integer greater than or equal to the corresponding element in the original tensor.
    ///
    /// # Arguments
    ///
    /// This function takes no arguments.
    ///
    /// # Returns
    ///
    /// * A new tensor where each element is the ceiling of the corresponding element in the original tensor.
    ///
    /// # Panics
    ///
    /// * This function should not panic under normal conditions.
    /// # Examples
    /// ```
    /// use tensor_dyn::tensor::Tensor;
    /// use tensor_dyn::NormalUaryOps;
    /// let a = Tensor::<f64>::new([0.1, 1.5, 2.9, 3.0]);
    /// let b = a.ceil().unwrap();
    /// ```
    fn ceil(&self) -> Result<Self::Output> {
        Ok(_Tensor::ceil(self)?.into())
    }

    fn ceil_<U>(&self, out: U) -> Result<Self::Output> where U: Borrow<Self::InplaceOutput> {
        Ok(_Tensor::ceil_(self, out)?.into())
    }

    /// Computes the element-wise sign of the tensor.
    ///
    /// This function returns a new tensor where each element represents the sign of the corresponding element in the original tensor:
    ///
    /// * `1` for positive values
    /// * `0` for zero
    /// * `-1` for negative values
    ///
    /// # Arguments
    ///
    /// This function takes no arguments.
    ///
    /// # Returns
    ///
    /// * A new tensor where each element is the sign of the corresponding element in the original tensor.
    ///
    /// # Panics
    ///
    /// * This function should not panic under normal conditions.
    /// # Examples
    /// ```
    /// use tensor_dyn::tensor::Tensor;
    /// use tensor_dyn::NormalUaryOps;
    /// let a = Tensor::<f64>::new([-1.0, 1.5, -2.9, 3.0]);
    /// let b = a.sign().unwrap();
    /// ```
    fn sign(&self) -> Result<Self::Output> {
        Ok(_Tensor::sign(self)?.into())
    }

    fn sign_<U>(&self, out: U) -> Result<Self::Output> where U: Borrow<Self::InplaceOutput> {
        Ok(_Tensor::sign_(self, out)?.into())
    }

    /// Clips (limits) the values of the tensor between the specified `min` and `max`.
    ///
    /// This function returns a new tensor where each element is clipped to be within the range `[min, max]`. If an element is less than `min`, it is set to `min`. If it is greater than `max`, it is set to `max`.
    ///
    /// # Arguments
    ///
    /// * `min` - The minimum allowed value for each element.
    /// * `max` - The maximum allowed value for each element.
    ///
    /// # Returns
    ///
    /// * A new tensor where each element is clipped to the specified range.
    ///
    /// # Panics
    ///
    /// * This function will panic if `min` is greater than `max`.
    /// # Examples
    /// ```
    /// use tensor_dyn::tensor::Tensor;
    /// use tensor_dyn::NormalUaryOps;
    /// let a = Tensor::<f64>::new([-1.0, 1.5, -2.9, 3.0]);
    /// let b = a.clip(-1.0, 1.0).unwrap();
    /// ```
    fn clamp(&self, min: Self::OutputMeta, max: Self::OutputMeta) -> Result<Self::Output> {
        Ok(_Tensor::clamp(self, min, max)?.into())
    }

    fn clamp_<U>(&self, min: Self::OutputMeta, max: Self::OutputMeta, out: U) -> Result<Self::Output>
        where U: Borrow<Self::InplaceOutput>
    {
        Ok(_Tensor::clamp_(self, min, max, out)?.into())
    }

    /// Computes the element-wise rounding of the tensor.
    ///
    /// This function rounds each element in the tensor to the nearest integer, returning a new tensor
    /// where each element is the nearest integer to the corresponding element in the original tensor.
    ///
    /// # Arguments
    ///
    /// This function takes no arguments.
    ///
    /// # Returns
    ///
    /// * A new tensor where each element is rounded to the nearest integer.
    ///
    /// # Panics
    ///
    /// * This function should not panic under normal conditions.
    /// # Examples
    /// ```
    /// use tensor_dyn::tensor::Tensor;
    /// use tensor_dyn::NormalUaryOps;
    /// let a = Tensor::<f64>::new([0.1, 1.5, 2.9, 3.0]);
    /// let b = a.round().unwrap();
    /// ```
    fn round(&self) -> Result<Self::Output> {
        Ok(_Tensor::round(self)?.into())
    }

    fn round_<U>(&self, out: U) -> Result<Self::Output> where U: Borrow<Self::InplaceOutput> {
        Ok(_Tensor::round_(self, out)?.into())
    }

    fn neg(&self) -> Result<Self> {
        Ok(_Tensor::neg(self)?.into())
    }

    fn neg_<U>(&self, out: U) -> Result<Self> where U: Borrow<Self::InplaceOutput> {
        Ok(_Tensor::neg_(self, out)?.into())
    }

    fn relu(&self) -> Result<Self::Output> {
        Ok(_Tensor::relu(self)?.into())
    }

    fn relu_<U>(&self, out: U) -> Result<Self::Output> where U: Borrow<Self::InplaceOutput> {
        Ok(_Tensor::relu_(self, out)?.into())
    }

    fn leaky_relu(&self, alpha: Self::OutputMeta) -> Result<Self::Output> {
        Ok(_Tensor::leaky_relu(self, alpha)?.into())
    }

    fn leaky_relu_<U>(&self, alpha: Self::OutputMeta, out: U) -> Result<Self::Output>
        where U: Borrow<Self::InplaceOutput>
    {
        Ok(_Tensor::leaky_relu_(self, alpha, out)?.into())
    }

    fn relu6(&self) -> Result<Self::Output> {
        Ok(_Tensor::relu6(self)?.into())
    }

    fn relu6_<U>(&self, out: U) -> Result<Self::Output> where U: Borrow<Self::InplaceOutput> {
        Ok(_Tensor::relu6_(self, out)?.into())
    }
}
