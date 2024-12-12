use crate::arch_simd::sleef::arch::helper_sse::vabs_vf_vf;
use crate::arch_simd::sleef::libm::sleefsimdsp::{xacosf_u1, xacoshf, xasinf_u1, xasinhf, xatan2f_u1, xatanf_u1, xatanhf, xcbrtf_u1, xcosf_u1, xcoshf, xerff_u1, xexp10f, xexp2f, xexpf, xexpm1f, xhypotf_u05, xlog10f, xlog1pf, xlog2f, xlogf_u1, xmaxf, xminf, xpowf, xroundf, xsincosf_u1, xsinf_u1, xsinhf, xsqrtf_u05, xtanf_u1, xtanhf, xtruncf};
use crate::traits::{ SimdMath, SimdSelect, VecTrait };
use crate::vectors::arch_simd::_128bit::u32x4::u32x4;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// a vector of 4 f32 values
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(C, align(16))]
pub struct f32x4(pub(crate) __m128);

impl PartialEq for f32x4 {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let cmp = _mm_cmpeq_ps(self.0, other.0);
            _mm_movemask_ps(cmp) == -1
        }
    }
}

impl Default for f32x4 {
    fn default() -> Self {
        unsafe { f32x4(_mm_setzero_ps()) }
    }
}

impl VecTrait<f32> for f32x4 {
    const SIZE: usize = 4;
    type Base = f32;
    #[inline(always)]
    fn copy_from_slice(&mut self, slice: &[f32]) {
        unsafe {
            _mm_storeu_ps(&mut self.0 as *mut _ as *mut f32, _mm_loadu_ps(slice.as_ptr()));
        }
    }
    #[inline(always)]
    fn mul_add(self, a: Self, b: Self) -> Self {
        #[cfg(all(target_arch = "x86_64", not(target_feature = "fma")))]
        unsafe {
            f32x4(_mm_add_ps(_mm_mul_ps(self.0, a.0), b.0))
        }
        #[cfg(all(target_arch = "x86_64", target_feature = "fma"))]
        unsafe {
            f32x4(_mm_fmadd_ps(self.0, a.0, b.0))
        }
    }
    #[inline(always)]
    fn sum(&self) -> f32 {
        unsafe {
            let sum = _mm_hadd_ps(self.0, self.0);
            _mm_cvtss_f32(_mm_hadd_ps(sum, sum))
        }
    }
    fn splat(val: f32) -> f32x4 {
        unsafe { f32x4(_mm_set1_ps(val)) }
    }
}

impl SimdSelect<f32x4> for u32x4 {
    fn select(&self, true_val: f32x4, false_val: f32x4) -> f32x4 {
        unsafe { f32x4(_mm_blendv_ps(false_val.0, true_val.0, std::mem::transmute(self.0))) }
    }
}
impl std::ops::Add for f32x4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        unsafe { f32x4(_mm_add_ps(self.0, rhs.0)) }
    }
}

impl std::ops::Sub for f32x4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        unsafe { f32x4(_mm_sub_ps(self.0, rhs.0)) }
    }
}

impl std::ops::Mul for f32x4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        unsafe { f32x4(_mm_mul_ps(self.0, rhs.0)) }
    }
}

impl std::ops::Div for f32x4 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        unsafe { f32x4(_mm_div_ps(self.0, rhs.0)) }
    }
}

impl std::ops::Rem for f32x4 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        unsafe {
            let a: [f32; 4] = std::mem::transmute(self.0);
            let b: [f32; 4] = std::mem::transmute(rhs.0);
            let c: [f32; 4] = [a[0] % b[0], a[1] % b[1], a[2] % b[2], a[3] % b[3]];
            f32x4(std::mem::transmute(c))
        }
    }
}
impl std::ops::Neg for f32x4 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        unsafe { f32x4(_mm_xor_ps(self.0, _mm_set1_ps(-0.0))) }
    }
}

impl SimdMath<f32> for f32x4 {
    fn sin(self) -> Self {
        f32x4(unsafe { xsinf_u1(self.0) })
    }
    fn cos(self) -> Self {
        f32x4(unsafe { xcosf_u1(self.0) })
    }
    fn tan(self) -> Self {
        f32x4(unsafe { xtanf_u1(self.0) })
    }

    fn square(self) -> Self {
        f32x4(unsafe { _mm_mul_ps(self.0, self.0) })
    }

    fn sqrt(self) -> Self {
        f32x4(unsafe { xsqrtf_u05(self.0) })
    }

    fn abs(self) -> Self {
        f32x4(unsafe { vabs_vf_vf(self.0) })
    }

    fn floor(self) -> Self {
        f32x4(unsafe { _mm_floor_ps(self.0) })
    }

    fn ceil(self) -> Self {
        f32x4(unsafe { _mm_ceil_ps(self.0) })
    }

    fn neg(self) -> Self {
        f32x4(unsafe { _mm_sub_ps(_mm_setzero_ps(), self.0) })
    }

    fn round(self) -> Self {
        f32x4(unsafe { xroundf(self.0) })
    }

    fn sign(self) -> Self {
        f32x4(unsafe { _mm_and_ps(self.0, _mm_set1_ps(0.0f32)) })
    }

    fn leaky_relu(self, _: f32) -> Self {
        todo!()
    }

    fn relu(self) -> Self {
        f32x4(unsafe { _mm_max_ps(self.0, _mm_setzero_ps()) })
    }

    fn relu6(self) -> Self {
        f32x4(unsafe { _mm_min_ps(self.relu().0, _mm_set1_ps(6.0f32)) })
    }

    fn pow(self, exp: Self) -> Self {
        f32x4(unsafe { xpowf(self.0, exp.0) })
    }

    fn asin(self) -> Self {
        f32x4(unsafe { xasinf_u1(self.0) })
    }

    fn acos(self) -> Self {
        f32x4(unsafe { xacosf_u1(self.0) })
    }

    fn atan(self) -> Self {
        f32x4(unsafe { xatanf_u1(self.0) })
    }

    fn sinh(self) -> Self {
        f32x4(unsafe { xsinhf(self.0) })
    }

    fn cosh(self) -> Self {
        f32x4(unsafe { xcoshf(self.0) })
    }

    fn tanh(self) -> Self {
        f32x4(unsafe { xtanhf(self.0) })
    }

    fn asinh(self) -> Self {
        f32x4(unsafe { xasinhf(self.0) })
    }

    fn acosh(self) -> Self {
        f32x4(unsafe { xacoshf(self.0) })
    }

    fn atanh(self) -> Self {
        f32x4(unsafe { xatanhf(self.0) })
    }

    fn exp2(self) -> Self {
        f32x4(unsafe { xexp2f(self.0) })
    }

    fn exp10(self) -> Self {
        f32x4(unsafe { xexp10f(self.0) })
    }

    fn expm1(self) -> Self {
        f32x4(unsafe { xexpm1f(self.0) })
    }

    fn log10(self) -> Self {
        f32x4(unsafe { xlog10f(self.0) })
    }

    fn log2(self) -> Self {
        f32x4(unsafe { xlog2f(self.0) })
    }

    fn log1p(self) -> Self {
        f32x4(unsafe { xlog1pf(self.0) })
    }

    fn hypot(self, other: Self) -> Self {
        f32x4(unsafe { xhypotf_u05(self.0, other.0) })
    }

    fn trunc(self) -> Self {
        f32x4(unsafe { xtruncf(self.0) })
    }

    fn erf(self) -> Self {
        f32x4(unsafe { xerff_u1(self.0) })
    }

    fn cbrt(self) -> Self {
        f32x4(unsafe { xcbrtf_u1(self.0) })
    }

    fn exp(self) -> Self {
        f32x4(unsafe { xexpf(self.0) })
    }

    fn ln(self) -> Self {
        f32x4(unsafe { xlogf_u1(self.0) })
    }

    fn log(self) -> Self {
        f32x4(unsafe { xlogf_u1(self.0) })
    }

    fn sincos(self) -> (Self, Self) {
        let ret = unsafe { xsincosf_u1(self.0) };
        (f32x4(ret.x), f32x4(ret.y))
    }

    fn atan2(self, other: Self) -> Self {
        f32x4(unsafe { xatan2f_u1(self.0, other.0) })
    }

    fn min(self, other: Self) -> Self {
        f32x4(unsafe { xminf(self.0, other.0) })
    }

    fn max(self, other: Self) -> Self {
        f32x4(unsafe { xmaxf(self.0, other.0) })
    }
}