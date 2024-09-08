use std::ops::{ Deref, DerefMut };

use crate::{into_vec::IntoVec, traits::{Init, VecSize, VecTrait}};

#[allow(non_camel_case_types)]
#[derive(Default, Clone, Copy, PartialEq)]
pub struct u8x16(pub(crate) std::simd::u8x16);

impl Deref for u8x16 {
    type Target = std::simd::u8x16;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for u8x16 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl VecTrait<u8> for u8x16 {
    #[inline(always)]
    fn copy_from_slice(&mut self, slice: &[u8]) {
        self.as_mut_array().copy_from_slice(slice);
    }
    #[inline(always)]
    fn as_ptr(&self) -> *const u8 {
        self.as_array().as_ptr()
    }
    #[inline(always)]
    fn _mul_add(self, a: Self, b: Self) -> Self {
        Self(self.0 * a.0 + b.0)
    }
    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.as_mut_array().as_mut_ptr()
    }
    #[inline(always)]
    fn as_mut_ptr_uncheck(&self) -> *mut u8 {
        self.as_array().as_ptr() as *mut _
    }
    #[inline(always)]
    fn sum(&self) -> u8 {
        self.as_array().iter().sum()
    }
    
    fn extract(self, idx: usize) -> u8 {
        self.as_array()[idx]
    }
}
impl VecSize for u8x16 {
    const SIZE: usize = 16;
}
impl Init<u8> for u8x16 {
    fn splat(val: u8) -> u8x16 {
        u8x16(std::simd::u8x16::splat(val))
    }

    unsafe fn from_ptr(ptr: *const u8) -> Self {
        unsafe { std::mem::transmute(std::arch::x86_64::_mm_loadu_si128(ptr as *const _)) }
    }
}
impl IntoVec<u8x16> for u8x16 {
    fn into_vec(self) -> u8x16 {
        self
    }
}
impl std::ops::Add for u8x16 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        u8x16(self.0 + rhs.0)
    }
}
impl std::ops::Sub for u8x16 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        u8x16(self.0 - rhs.0)
    }
}
impl std::ops::Mul for u8x16 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        u8x16(self.0 * rhs.0)
    }
}
impl std::ops::Div for u8x16 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        u8x16(self.0 / rhs.0)
    }
}
impl std::ops::Rem for u8x16 {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self {
        u8x16(self.0 % rhs.0)
    }
}