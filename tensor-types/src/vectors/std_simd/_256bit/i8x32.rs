use std::ops::{ Deref, DerefMut };

use crate::vectors::traits::VecTrait;

/// a vector of 32 i8 values
#[allow(non_camel_case_types)]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
#[repr(C, align(32))]
pub struct i8x32(pub(crate) std::simd::i8x32);

impl Deref for i8x32 {
    type Target = std::simd::i8x32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for i8x32 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl VecTrait<i8> for i8x32 {
    const SIZE: usize = 32;
    type Base = i8;
    #[inline(always)]
    fn mul_add(self, a: Self, b: Self) -> Self {
        Self(self.0 * a.0 + b.0)
    }
    #[inline(always)]
    fn copy_from_slice(&mut self, slice: &[i8]) {
        self.as_mut_array().copy_from_slice(slice);
    }
    #[inline(always)]
    fn sum(&self) -> i8 {
        self.as_array().iter().sum()
    }
    fn splat(val: i8) -> i8x32 {
        i8x32(std::simd::i8x32::splat(val))
    }
}

impl std::ops::Add for i8x32 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        i8x32(self.0 + rhs.0)
    }
}
impl std::ops::Sub for i8x32 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        i8x32(self.0 - rhs.0)
    }
}
impl std::ops::Mul for i8x32 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        i8x32(self.0 * rhs.0)
    }
}
impl std::ops::Div for i8x32 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        i8x32(self.0 / rhs.0)
    }
}
impl std::ops::Rem for i8x32 {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        i8x32(self.0 % rhs.0)
    }
}
impl std::ops::Neg for i8x32 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        i8x32(-self.0)
    }
}