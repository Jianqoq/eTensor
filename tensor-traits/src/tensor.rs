use std::ops::{ Div, Sub };

use tensor_common::{ layout::Layout, pointer::Pointer, shape::Shape, strides::Strides };
use tensor_types::{
    convertion::{ Convertor, FromScalar },
    dtype::TypeCommon,
    into_scalar::IntoScalar,
    type_promote::{ FloatOut, NormalOut },
};

pub trait TensorInfo<T> {
    fn ptr(&self) -> Pointer<T>;
    fn size(&self) -> usize;
    fn shape(&self) -> &Shape;
    fn strides(&self) -> &Strides;
    fn layout(&self) -> &Layout;
    fn parent(&self) -> Option<Pointer<T>>;
    fn ndim(&self) -> usize;
    fn is_contiguous(&self) -> bool;
    fn elsize() -> usize {
        std::mem::size_of::<T>()
    }
}

pub trait TensorLike<T, OutputMeta = T, Output = Self> {
    type Output;
    fn to_raw(&self) -> &[T];
    fn to_raw_mut(&self) -> &mut [T];
    fn elsize() -> usize {
        std::mem::size_of::<T>()
    }
    fn static_cast(&self) -> anyhow::Result<Self::Output>;
}

pub trait TensorCreator<T, Output = Self> where Self: Sized {
    type StridedIter;
    type Mask;
    type Basic;

    /// Creates an empty tensor with the specified shape.
    ///
    /// This function generates a tensor with a given shape, but without initializing its values.
    /// It is the only method to create a tensor in an uninitialized state.
    ///
    /// # Arguments
    /// - `shape`: The shape of the tensor to be created.
    ///
    /// # Returns
    /// `Result<Output>`: An empty tensor with the specified shape.
    ///
    /// # Examples
    /// ```
    /// let empty_tensor = YourType::empty([2, 3]); // Creates a 2x3 empty tensor
    /// ```
    fn empty<S: Into<Shape>>(shape: S) -> anyhow::Result<Output>;

    /// Creates a tensor filled with zeros.
    ///
    /// This function generates a tensor of a given shape, where each element is initialized to zero.
    ///
    /// # Arguments
    /// - `shape`: The shape of the tensor to be created.
    ///
    /// # Returns
    /// `Result<Output>`: A tensor filled with zeros.
    ///
    /// # Examples
    /// ```
    /// let zeros_tensor = YourType::zeros([2, 3]); // Creates a 2x3 tensor filled with zeros
    /// ```
    fn zeros<S: Into<Shape>>(shape: S) -> anyhow::Result<Output>;

    /// Creates a tensor filled with ones.
    ///
    /// This function generates a tensor of a given shape, where each element is initialized to one.
    ///
    /// # Arguments
    /// - `shape`: The shape of the tensor to be created.
    ///
    /// # Returns
    /// `Result<Output>`: A tensor filled with ones.
    ///
    /// # Examples
    /// ```
    /// let ones_tensor = YourType::ones([2, 3]); // Creates a 2x3 tensor filled with ones
    /// ```
    fn ones<S: Into<Shape>>(shape: S) -> anyhow::Result<Output> where u8: IntoScalar<T>;

    /// Creates a tensor with the same shape as the caller tensor, but empty.
    ///
    /// This function generates an empty tensor (uninitialized values) having the same shape as the provided tensor.
    ///
    /// # Returns
    /// `Result<Output>`: An empty tensor with the same shape as `self`.
    ///
    /// # Examples
    /// ```
    /// let original_tensor = YourType::new(...);
    /// let empty_tensor = original_tensor.empty_like(); // New tensor with the same shape, but empty
    /// ```
    fn empty_like(&self) -> anyhow::Result<Output>;

    /// Creates a tensor with all zeros, based on the shape of `self`.
    ///
    /// This function generates a tensor filled with zeros having the same shape as the provided tensor.
    ///
    /// # Returns
    /// `Result<Output>`: A tensor filled with zeros, having the same shape as `self`.
    ///
    /// # Examples
    /// ```
    /// let original_tensor = YourType::new(...);
    /// let zeros_tensor = original_tensor.zeros_like(); // New tensor with the same shape, filled with zeros
    /// ```
    fn zeros_like(&self) -> anyhow::Result<Output>;

    /// Creates a tensor with all ones, based on the shape of `self`.
    ///
    /// This function generates a tensor filled with ones having the same shape as the provided tensor.
    ///
    /// # Returns
    /// `Result<Output>`: A tensor filled with ones, having the same shape as `self`.
    ///
    /// # Examples
    /// ```
    /// let original_tensor = YourType::new(...);
    /// let ones_tensor = original_tensor.ones_like(); // New tensor with the same shape, filled with ones
    /// ```
    fn ones_like(&self) -> anyhow::Result<Output> where u8: IntoScalar<T>;

    /// Creates a tensor filled entirely with a specified value.
    ///
    /// This function generates a tensor of a given shape, where each element is set to the specified value.
    ///
    /// # Type Parameters
    /// - `S`: A type that can be converted into the `Shape` type.
    ///
    /// # Arguments
    /// - `val`: The value to fill the tensor with.
    /// - `shape`: The shape of the tensor to be created.
    ///
    /// # Returns
    /// `Result<Output>`: The tensor filled with the specified value.
    ///
    /// # Examples
    /// ```
    /// let tensor = YourType::full(3.14, [2, 2]); // Creates a 2x2 tensor filled with 3.14
    /// ```
    fn full<S: Into<Shape>>(val: T, shape: S) -> anyhow::Result<Output>;

    /// Creates a tensor with the same shape as another tensor, filled with a specified value.
    ///
    /// This method generates a new tensor having the same shape as `self`,
    /// but with each element set to the specified value.
    ///
    /// # Arguments
    /// - `val`: The value to fill the new tensor with.
    ///
    /// # Returns
    /// `Result<Output>`: A new tensor with the same shape as `self`, filled with `val`.
    ///
    /// # Examples
    /// ```
    /// let original_tensor = YourType::new(...);
    /// let filled_tensor = original_tensor.full_like(1.0); // New tensor with the same shape, filled with 1.0
    /// ```
    fn full_like(&self, val: T) -> anyhow::Result<Output>;

    /// Creates a tensor with a range of values from `start` to `end` (exclusive).
    ///
    /// The function generates a one-dimensional tensor containing a sequence of values
    /// starting from `start` and ending before `end`.
    ///
    /// # Type Constraints
    /// - `T`: Must be convertible to `usize` and support basic arithmetic operations.
    ///
    /// # Arguments
    /// - `start`: The starting value of the range.
    /// - `end`: The end value of the range (exclusive).
    ///
    /// # Returns
    /// `Result<Output>`: A tensor containing the range of values.
    ///
    /// # Examples
    /// ```
    /// let range_tensor = YourType::arange(0, 10); // Creates a tensor with values from 0 to 9
    /// ```
    fn arange<U>(start: U, end: U) -> anyhow::Result<Output>
        where T: Convertor + FromScalar<usize> + FromScalar<U> + NormalOut<T, Output = T>;

    /// Creates a tensor with a range of values from `start` to `end` (exclusive), using a specified step.
    ///
    /// This function generates a one-dimensional tensor containing a sequence of values,
    /// starting from `start`, incrementing by `step`, and ending before `end`.
    ///
    /// # Arguments
    /// - `start`: The starting value of the range.
    /// - `end`: The end value of the range (exclusive).
    /// - `step`: The step value to increment by.
    ///
    /// # Returns
    /// `Result<Output>`: A tensor containing the range of values with the specified step.
    ///
    /// # Examples
    /// ```
    /// let range_step_tensor = YourType::arange_step(0, 10, 2); // Creates a tensor with values [0, 2, 4, 6, 8]
    /// ```
    fn arange_step(start: T, end: T, step: T) -> anyhow::Result<Output>
        where T: Convertor + FromScalar<usize> + NormalOut<T, Output = T>;

    /// Creates an identity matrix of size `n` x `m`, with ones on the k-th diagonal and zeros elsewhere.
    ///
    /// # Arguments
    /// - `n`: The number of rows in the matrix.
    /// - `m`: The number of columns in the matrix.
    /// - `k`: The index of the diagonal. A positive value refers to an upper diagonal,
    ///        a negative value to a lower diagonal, and zero to the main diagonal.
    ///
    /// # Returns
    /// `anyhow::Result<Output>`: The identity matrix as specified.
    ///
    /// # Examples
    /// ```
    /// let eye_matrix = Tensor::<i32>::eye(3, 3, 0); // Creates a 3x3 identity matrix
    /// ```
    fn eye(n: usize, m: usize, k: usize) -> anyhow::Result<Output> where u8: IntoScalar<T>;

    /// Returns evenly spaced numbers over a specified interval.
    ///
    /// Generates `num` evenly spaced samples, calculated over the interval [start, end].
    /// The endpoint of the interval can optionally be excluded.
    ///
    /// # Arguments
    /// - `start`: The starting value of the sequence.
    /// - `end`: The end value of the sequence.
    /// - `num`: The number of evenly spaced samples to generate.
    /// - `include_end`: Whether to include the end value in the sequence.
    ///
    /// # Returns
    /// `Result<Output>`: A tensor with the evenly spaced numbers.
    ///
    /// # Examples
    /// ```
    /// let linspace_tensor = YourType::linspace(0., 10., 5, false);
    /// // Creates a tensor with values [0., 2.5, 5., 7.5, 10.]
    /// ```
    fn linspace(start: T, end: T, num: usize, include_end: bool) -> anyhow::Result<Output>
        where
            T: Convertor +
                num::Float +
                FromScalar<usize> +
                FromScalar<f64> +
                NormalOut<T, Output = T>;

    /// Returns numbers spaced evenly on a log scale.
    ///
    /// Generates `num` samples, evenly spaced on a log scale. The sequence starts at `base ** start` and ends with `base ** end`.
    /// The endpoint of the interval can optionally be excluded.
    ///
    /// # Arguments
    /// - `start`: The exponent of the starting value.
    /// - `end`: The exponent of the end value.
    /// - `num`: The number of samples to generate.
    /// - `include_end`: Whether to include the end value in the sequence.
    /// - `base`: The base of the logarithm.
    ///
    /// # Returns
    /// `Result<Output>`: A tensor with the numbers spaced evenly on a log scale.
    ///
    /// # Examples
    /// ```
    /// let logspace_tensor = YourType::logspace(0., 10., 5, false, 2.);
    /// // Creates a tensor with values [1., 2.160119483, 4.641588833, 10., 21.5443469]
    /// ```
    fn logspace(start: T, end: T, num: usize, include_end: bool, base: T) -> anyhow::Result<Output>
        where
            T: Convertor +
                num::Float +
                FromScalar<usize> +
                FromScalar<f64> +
                NormalOut<T, Output = T>;

    /// Returns numbers spaced evenly on a geometric scale.
    ///
    /// Generates `num` samples, evenly spaced on a geometric scale over the interval [start, end].
    /// The endpoint of the interval can optionally be included.
    ///
    /// # Arguments
    /// - `start`: The starting value of the sequence.
    /// - `end`: The end value of the sequence.
    /// - `n`: The number of samples to generate.
    /// - `include_end`: Whether to include the end value in the sequence.
    ///
    /// # Returns
    /// `Result<Output>`: A tensor with the numbers spaced evenly on a geometric scale.
    ///
    /// # Type Constraints
    /// - `T`: Must be convertible to `f64` and `usize`, and support floating-point arithmetic and comparison.
    ///
    /// # Examples
    /// ```
    /// let geomspace_tensor = YourType::geomspace(1., 1000., 4, true);
    /// // Creates a tensor with values [1., 10., 100., 1000.]
    /// ```
    fn geomspace(start: T, end: T, n: usize, include_end: bool) -> anyhow::Result<Output>
        where
            T: PartialOrd +
                FloatOut<T> +
                NormalOut<T, Output = T> +
                FromScalar<<T as FloatOut>::Output> +
                std::ops::Neg<Output = T>,
            <T as FloatOut<T>>::Output: Sub<Output = <T as FloatOut>::Output> +
                FromScalar<usize> +
                FromScalar<f64> +
                Div<Output = <T as FloatOut>::Output> +
                NormalOut<Output = <T as FloatOut>::Output> + CommonBounds;

    /// Creates a triangular matrix with dimensions `n` x `m`.
    ///
    /// This function generates a matrix of size `n` x `m`, filled with ones below (`low_triangle` = true)
    /// or above (`low_triangle` = false) the k-th diagonal.
    ///
    /// # Arguments
    /// - `n`: The number of rows in the matrix.
    /// - `m`: The number of columns in the matrix.
    /// - `k`: The index of the diagonal.
    /// - `low_triangle`: Whether to create a lower triangular matrix (true) or upper triangular matrix (false).
    ///
    /// # Returns
    /// `anyhow::Result<Output>`: The triangular matrix as specified.
    ///
    /// # Examples
    /// ```
    /// let tri_matrix = YourType::tri(3, 3, 0, true); // Creates a 3x3 lower triangular matrix
    /// ```
    fn tri(n: usize, m: usize, k: i64, low_triangle: bool) -> anyhow::Result<Output>
        where u8: IntoScalar<T>;

    /// Creates a lower triangular matrix from the existing tensor.
    ///
    /// The lower triangular part of the tensor is retained, and elements above the k-th diagonal are set to zero.
    ///
    /// # Arguments
    /// - `k`: The index of the diagonal. Elements above this diagonal are set to zero.
    ///
    /// # Returns
    /// `anyhow::Result<Output>`: The lower triangular matrix.
    ///
    /// # Examples
    /// ```
    /// let tensor = YourType::new(...);
    /// let lower_tri_matrix = tensor.tril(0); // Creates a lower triangular matrix from tensor
    /// ```
    fn tril(&self, k: i64) -> anyhow::Result<Self>;

    /// Creates an upper triangular matrix from the existing tensor.
    ///
    /// The upper triangular part of the tensor is retained, and elements below the k-th diagonal are set to zero.
    ///
    /// # Arguments
    /// - `k`: The index of the diagonal. Elements below this diagonal are set to zero.
    ///
    /// # Returns
    /// `anyhow::Result<Output>`: The upper triangular matrix.
    ///
    /// # Type Constraints
    /// - `Output`: The output type must support multiplication with `Self::Mask`.
    ///
    /// # Examples
    /// ```
    /// let tensor = YourType::new(...);
    /// let upper_tri_matrix = tensor.triu(0); // Creates an upper triangular matrix from tensor
    /// ```
    fn triu(&self, k: i64) -> anyhow::Result<Self>;

    /// Creates an identity matrix of size `n` x `n`.
    ///
    /// This function generates an identity matrix with ones on the main diagonal and zeros elsewhere.
    ///
    /// # Arguments
    /// - `n`: The size of the matrix (both number of rows and columns).
    ///
    /// # Returns
    /// `Result<Output>`: The identity matrix of size `n` x `n`.
    ///
    /// # Examples
    /// ```
    /// let identity_matrix = YourType::identity(3); // Creates a 3x3 identity matrix
    /// ```
    fn identity(n: usize) -> anyhow::Result<Output> where u8: IntoScalar<T>;
}

pub trait TensorAlloc<Output = Self> {
    type Meta;
    fn _empty<S: Into<Shape>>(shape: S) -> anyhow::Result<Output> where Self: Sized;
}

pub trait CommonBounds: Sync + Send + Clone + Copy + TypeCommon + 'static {}
impl<T: Sync + Send + Clone + Copy + TypeCommon + 'static> CommonBounds for T {}
