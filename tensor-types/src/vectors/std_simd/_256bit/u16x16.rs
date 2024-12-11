use std::ops::{ Deref, DerefMut };
use crate::vectors::traits::VecTrait;

/// a vector of 16 u16 values
#[allow(non_camel_case_types)]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
#[repr(C, align(32))]
pub struct u16x16(pub(crate) std::simd::u16x16);

impl Deref for u16x16 {
    type Target = std::simd::u16x16;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for u16x16 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl VecTrait<u16> for u16x16 {
    const SIZE: usize = 16;
    type Base = u16;
    #[inline(always)]
    fn mul_add(self, a: Self, b: Self) -> Self {
        Self(self.0 * a.0 + b.0)
    }
    #[inline(always)]
    fn copy_from_slice(&mut self, slice: &[u16]) {
        self.as_mut_array().copy_from_slice(slice);
    }
    #[inline(always)]
    fn sum(&self) -> u16 {
        self.as_array().iter().sum()
    }
    fn splat(val: u16) -> u16x16 {
        u16x16(std::simd::u16x16::splat(val))
    }
}

impl std::ops::Add for u16x16 {
    type Output = u16x16;
    fn add(self, rhs: Self) -> Self::Output {
        u16x16(self.0 + rhs.0)
    }
}
impl std::ops::Sub for u16x16 {
    type Output = u16x16;
    fn sub(self, rhs: Self) -> Self::Output {
        u16x16(self.0 - rhs.0)
    }
}
impl std::ops::Mul for u16x16 {
    type Output = u16x16;
    fn mul(self, rhs: Self) -> Self::Output {
        u16x16(self.0 * rhs.0)
    }
}
impl std::ops::Div for u16x16 {
    type Output = u16x16;
    fn div(self, rhs: Self) -> Self::Output {
        u16x16(self.0 / rhs.0)
    }
}
impl std::ops::Rem for u16x16 {
    type Output = u16x16;
    fn rem(self, rhs: Self) -> Self::Output {
        u16x16(self.0 % rhs.0)
    }
}