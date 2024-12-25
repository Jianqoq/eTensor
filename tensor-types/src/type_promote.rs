use crate::convertion::Convertor;
use crate::convertion::VecConvertor;
use crate::dtype::FloatConst;
use crate::dtype::TypeCommon;
#[cfg(any(
    all(not(target_feature = "avx2"), target_feature = "sse"),
    target_arch = "arm",
    target_arch = "aarch64",
    target_feature = "neon"
))]
use crate::simd::_128bit::*;
#[cfg(target_feature = "avx2")]
use crate::simd::_256bit::*;
#[cfg(target_feature = "avx512f")]
use crate::simd::_512bit::*;
use crate::traits::SimdMath;
use crate::vectors::traits::SimdCompare;
use crate::vectors::traits::SimdSelect;
use crate::vectors::traits::VecTrait;
use half::bf16;
use half::f16;
use num_complex::{Complex32, Complex64};
use num_traits::float::Float;
#[cfg(feature = "stdsimd")]
use sleef::Sleef;
use std::ops::Neg;
use tensor_macros::{
    float_out_binary, float_out_binary_simd_with_lhs_scalar, float_out_binary_simd_with_rhs_scalar,
    float_out_unary, impl_bitwise_out, impl_cmp, impl_eval, impl_normal_out_binary,
    impl_normal_out_simd, impl_normal_out_simd_with_lhs_scalar,
    impl_normal_out_simd_with_rhs_scalar, impl_normal_out_unary, impl_normal_out_unary_simd,
    simd_cmp, simd_eval, simd_float_out_unary,
};
#[cfg(feature = "cuda")]
mod cuda_imports {
    use super::*;
    use crate::cuda_types::convertion::CudaConvertor;
    use crate::cuda_types::scalar::Scalar;
    use tensor_macros::{
        float_out_binary_cuda, float_out_unary_cuda, impl_cmp_cuda, impl_cuda_bitwise_out,
        impl_cuda_normal_out_binary, impl_normal_out_unary_cuda,
    };
    float_out_binary_cuda!();
    impl_cuda_normal_out_binary!();
    impl_normal_out_unary_cuda!();
    impl_cuda_bitwise_out!();
    impl_cmp_cuda!();
    float_out_unary_cuda!();
}

use tensor_macros::{float_out_binary_simd, simd_bitwise};

/// this trait is used to perform type promotion in dynamic graph
pub trait FloatOutBinary<RHS = Self> {
    /// the output type
    type Output;
    /// perform a / b
    fn _div(self, rhs: RHS) -> Self::Output;
    /// perform log<sub>b</sub>(x)
    fn _log(self, base: RHS) -> Self::Output;
}

float_out_binary!();
float_out_binary_simd!();
float_out_binary_simd_with_rhs_scalar!();
float_out_binary_simd_with_lhs_scalar!();

/// this trait is used to perform normal operations that don't require type promotion
pub trait NormalOut<RHS = Self> {
    /// the output type
    type Output;
    /// perform a + b
    fn _add(self, rhs: RHS) -> Self::Output;
    /// perform a - b
    fn _sub(self, rhs: RHS) -> Self::Output;
    /// perform self * a + b, fused multiply add
    /// if the hardware supports it, it can speed up the calculation and reduce the rounding error
    fn _mul_add(self, a: RHS, b: RHS) -> Self::Output;
    /// perform a * b
    fn _mul(self, rhs: RHS) -> Self::Output;
    /// perform a<sup>b</sup>
    fn _pow(self, rhs: RHS) -> Self::Output;
    /// perform a % b
    fn _rem(self, rhs: RHS) -> Self::Output;
    /// perform max(x, y)
    fn _max(self, rhs: RHS) -> Self::Output;
    /// perform min(x, y)
    fn _min(self, rhs: RHS) -> Self::Output;
    /// restrict the value of x to the range [min, max]
    fn _clip(self, min: RHS, max: RHS) -> Self::Output;
}

impl_normal_out_binary!();

impl_normal_out_simd!();

impl_normal_out_simd_with_rhs_scalar!();

impl_normal_out_simd_with_lhs_scalar!();
/// this trait is used to perform normal unary operations that don't require type promotion
pub trait NormalOutUnary {
    /// the base type,
    type Base;
    /// perform x<sup>2</sup>
    fn _square(self) -> Self;
    /// perform |x|
    fn _abs(self) -> Self;
    /// perform &lceil;x&rceil;
    fn _ceil(self) -> Self;
    /// perform &lfloor;x&rfloor;
    fn _floor(self) -> Self;
    /// perform -x
    fn _neg(self) -> Self;
    /// perform rounding
    fn _round(self) -> Self;
    /// get the sign of x
    fn _sign(self) -> Self;

    /// Perform the leaky ReLU (Rectified Linear Unit) activation function.
    ///
    /// Formula: f(x) = x if x > 0 else alpha * x
    fn _leaky_relu(self, alpha: Self::Base) -> Self;

    /// Perform the ReLU (Rectified Linear Unit) activation function.
    ///
    /// Formula: f(x) = max(0, x)
    fn _relu(self) -> Self;

    /// Perform the ReLU6 activation function.
    ///
    /// Formula: f(x) = min(6, max(0, x))
    fn _relu6(self) -> Self;
}

impl_normal_out_unary!();

impl_normal_out_unary_simd!();

/// this trait is used to perform bitwise operations
pub trait BitWiseOut<RHS = Self> {
    /// the output type
    type Output;
    /// perform a & b
    fn _bitand(self, rhs: RHS) -> Self::Output;
    /// perform a | b
    fn _bitor(self, rhs: RHS) -> Self::Output;
    /// perform a ^ b
    fn _bitxor(self, rhs: RHS) -> Self::Output;
    /// perform !a
    fn _not(self) -> Self::Output;
    /// perform a << b
    fn _shl(self, rhs: RHS) -> Self::Output;
    /// perform a >> b
    fn _shr(self, rhs: RHS) -> Self::Output;
}

impl_bitwise_out!();

simd_bitwise!();

/// this trait is used to perform comparison operations
pub trait Cmp<RHS = Self> {
    /// the output type
    type Output;
    /// perform a == b
    fn _eq(self, rhs: RHS) -> Self::Output;
    /// perform a != b
    fn _ne(self, rhs: RHS) -> Self::Output;
    /// perform a < b
    fn _lt(self, rhs: RHS) -> Self::Output;
    /// perform a <= b
    fn _le(self, rhs: RHS) -> Self::Output;
    /// perform a > b
    fn _gt(self, rhs: RHS) -> Self::Output;
    /// perform a >= b
    fn _ge(self, rhs: RHS) -> Self::Output;
}

impl_cmp!();

/// this trait is used to perform comparison operations on simd
pub trait SimdCmp<RHS = Self> {
    /// the output type
    type Output;
    /// perform a == b, return a mask
    ///
    /// # Note
    ///
    /// The mask may not be a boolean value, the type is based on the byte width of the simd
    fn _eq(self, rhs: RHS) -> Self::Output;
    /// perform a != b, return a mask
    ///
    /// # Note
    ///
    /// The mask may not be a boolean value, the type is based on the byte width of the simd
    fn _ne(self, rhs: RHS) -> Self::Output;
    /// perform a < b, return a mask
    ///
    /// # Note
    ///
    /// The mask may not be a boolean value, the type is based on the byte width of the simd
    fn _lt(self, rhs: RHS) -> Self::Output;
    /// perform a <= b, return a mask
    ///
    /// # Note
    ///
    /// The mask may not be a boolean value, the type is based on the byte width of the simd
    fn _le(self, rhs: RHS) -> Self::Output;
    /// perform a > b, return a mask
    ///
    /// # Note
    ///
    /// The mask may not be a boolean value, the type is based on the byte width of the simd
    fn _gt(self, rhs: RHS) -> Self::Output;
    /// perform a >= b, return a mask
    ///
    /// # Note
    ///
    /// The mask may not be a boolean value, the type is based on the byte width of the simd
    fn _ge(self, rhs: RHS) -> Self::Output;
}

simd_cmp!();

/// this trait is used to perform evaluation operations
pub trait Eval {
    /// the output type
    type Output;
    /// check if the value is nan
    fn _is_nan(&self) -> Self::Output;
    /// check if the value is finite
    fn _is_true(&self) -> Self::Output;
    /// check if the value is infinite
    fn _is_inf(&self) -> Self::Output;
}

impl_eval!();
simd_eval!();

/// This trait is used to perform various unary floating-point operations.
pub trait FloatOutUnary {
    /// The output type.
    type Output;
    /// The base type, typically used for parameters like `alpha`.
    type Base;

    /// Perform the natural exponential function: e<sup>x</sup>.
    fn _exp(self) -> Self::Output;

    /// Perform the base-2 exponential function: 2<sup>x</sup>.
    fn _exp2(self) -> Self::Output;

    /// Perform the natural logarithm: ln(x).
    fn _ln(self) -> Self::Output;

    /// Perform the CELU (Continuously Differentiable Exponential Linear Unit) activation function.
    ///
    /// Formula: f(x) = max(0, x) + min(0, alpha * (e<sup>(x / alpha)</sup> - 1))
    fn _celu(self, alpha: Self::Base) -> Self::Output;

    /// Perform the base-2 logarithm: log<sub>2</sub>(x).
    fn _log2(self) -> Self::Output;

    /// Perform the base-10 logarithm: log<sub>10</sub>(x).
    fn _log10(self) -> Self::Output;

    /// Perform the square root: √x.
    fn _sqrt(self) -> Self::Output;

    /// Perform the sine function: sin(x).
    fn _sin(self) -> Self::Output;

    /// Perform the cosine function: cos(x).
    fn _cos(self) -> Self::Output;

    /// Perform the tangent function: tan(x).
    fn _tan(self) -> Self::Output;

    /// Perform the inverse sine (arcsin) function: asin(x).
    fn _asin(self) -> Self::Output;

    /// Perform the inverse cosine (arccos) function: acos(x).
    fn _acos(self) -> Self::Output;

    /// Perform the inverse tangent (arctan) function: atan(x).
    fn _atan(self) -> Self::Output;

    /// Perform the hyperbolic sine function: sinh(x).
    fn _sinh(self) -> Self::Output;

    /// Perform the hyperbolic cosine function: cosh(x).
    fn _cosh(self) -> Self::Output;

    /// Perform the hyperbolic tangent function: tanh(x).
    fn _tanh(self) -> Self::Output;

    /// Perform the inverse hyperbolic sine (arsinh) function: asinh(x).
    fn _asinh(self) -> Self::Output;

    /// Perform the inverse hyperbolic cosine (arcosh) function: acosh(x).
    fn _acosh(self) -> Self::Output;

    /// Perform the inverse hyperbolic tangent (artanh) function: atanh(x).
    fn _atanh(self) -> Self::Output;

    /// Perform the reciprocal function: 1 / x.
    fn _recip(self) -> Self::Output;

    /// Perform the error function (erf).
    fn _erf(self) -> Self::Output;

    /// Perform the sigmoid function: 1 / (1 + e<sup>-x</sup>).
    fn _sigmoid(self) -> Self::Output;

    /// Perform the ELU (Exponential Linear Unit) activation function.
    ///
    /// Formula: f(x) = x if x > 0 else alpha * (e<sup>x</sup> - 1)
    fn _elu(self, alpha: Self::Base) -> Self::Output;

    /// Perform the GELU (Gaussian Error Linear Unit) activation function.
    fn _gelu(self) -> Self::Output;

    /// Perform the SELU (Scaled Exponential Linear Unit) activation function.
    ///
    /// Formula: f(x) = scale * (x if x > 0 else alpha * (e<sup>x</sup> - 1))
    fn _selu(self, alpha: Self::Base, scale: Self::Base) -> Self::Output;

    /// Perform the hard sigmoid activation function.
    ///
    /// Formula: f(x) = min(1, max(0, 0.2 * x + 0.5))
    fn _hard_sigmoid(self) -> Self::Output;

    /// Perform the fast approximation of the hard sigmoid activation function.
    fn _fast_hard_sigmoid(self) -> Self::Output;

    /// Perform the hard swish activation function.
    ///
    /// Formula: f(x) = x * min(1, max(0, 0.2 * x + 0.5))
    fn _hard_swish(self) -> Self::Output;

    /// Perform the softplus activation function.
    ///
    /// Formula: f(x) = ln(1 + e<sup>x</sup>)
    fn _softplus(self) -> Self::Output;

    /// Perform the softsign activation function.
    ///
    /// Formula: f(x) = x / (1 + |x|)
    fn _softsign(self) -> Self::Output;

    /// Perform the mish activation function.
    ///
    /// Formula: f(x) = x * tanh(ln(1 + e<sup>x</sup>))
    fn _mish(self) -> Self::Output;

    /// Perform the cube root function: ∛x.
    fn _cbrt(self) -> Self::Output;
}

float_out_unary!();

simd_float_out_unary!();
