use std::ops::{ Deref, DerefMut };
use std::simd::num::SimdFloat;
use crate::vectors::traits::{ SimdSelect, VecTrait };
use std::simd::StdFloat;

/// a vector of 8 f32 values
#[allow(non_camel_case_types)]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
#[repr(C, align(32))]
pub struct f32x8(pub(crate) std::simd::f32x8);

impl Deref for f32x8 {
    type Target = std::simd::f32x8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for f32x8 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl VecTrait<f32> for f32x8 {
    const SIZE: usize = 8;
    type Base = f32;
    #[inline(always)]
    fn mul_add(self, a: Self, b: Self) -> Self {
        f32x8(self.0.mul_add(a.0, b.0))
    }
    #[inline(always)]
    fn copy_from_slice(&mut self, slice: &[f32]) {
        self.as_mut_array().copy_from_slice(slice)
    }
    #[inline(always)]
    fn sum(&self) -> f32 {
        self.reduce_sum()
    }
    fn splat(val: f32) -> f32x8 {
        f32x8(std::simd::f32x8::splat(val))
    }
}

impl SimdSelect<f32x8> for crate::vectors::std_simd::_256bit::u32x8::u32x8 {
    fn select(&self, true_val: f32x8, false_val: f32x8) -> f32x8 {
        let mask: std::simd::mask32x8 = unsafe { std::mem::transmute(*self) };
        f32x8(mask.select(true_val.0, false_val.0))
    }
}

impl std::ops::Add for f32x8 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        f32x8(self.0 + rhs.0)
    }
}

impl std::ops::Sub for f32x8 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        f32x8(self.0 - rhs.0)
    }
}

impl std::ops::Mul for f32x8 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        f32x8(self.0 * rhs.0)
    }
}

impl std::ops::Div for f32x8 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        f32x8(self.0 / rhs.0)
    }
}

impl std::ops::Rem for f32x8 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        f32x8(self.0 % rhs.0)
    }
}
impl std::ops::Neg for f32x8 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        f32x8(-self.0)
    }
}
