use std::simd::cmp::SimdPartialOrd;
use std::simd::{ cmp::SimdPartialEq, Simd };

use crate::traits::SimdCompare;
use crate::vectors::traits::VecTrait;

use super::i8x32::i8x32;

/// a vector of 32 bool values
#[allow(non_camel_case_types)]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
#[repr(C, align(32))]
pub struct boolx32(pub(crate) [bool; 32]);

impl VecTrait<bool> for boolx32 {
    const SIZE: usize = 32;
    type Base = bool;
    #[inline(always)]
    fn mul_add(self, _: Self, _: Self) -> Self {
        todo!()
    }
    #[inline(always)]
    fn copy_from_slice(&mut self, slice: &[bool]) {
        self.0.copy_from_slice(slice);
    }
    #[inline(always)]
    fn sum(&self) -> bool {
        self.0
            .iter()
            .map(|&x| x as u8)
            .sum::<u8>() > 0
    }
    fn splat(val: bool) -> boolx32 {
        boolx32([val; 32])
    }
}

impl SimdCompare for boolx32 {
    type SimdMask = i8x32;
    fn simd_eq(self, rhs: Self) -> i8x32 {
        let lhs: Simd<u8, 32> = unsafe { std::mem::transmute(self) };
        let rhs: Simd<u8, 32> = unsafe { std::mem::transmute(rhs) };
        i8x32(lhs.simd_eq(rhs).to_int())
    }
    fn simd_ne(self, rhs: Self) -> i8x32 {
        let lhs: Simd<u8, 32> = unsafe { std::mem::transmute(self) };
        let rhs: Simd<u8, 32> = unsafe { std::mem::transmute(rhs) };
        i8x32(lhs.simd_ne(rhs).to_int())
    }   
    fn simd_lt(self, rhs: Self) -> i8x32 {
        let lhs: Simd<u8, 32> = unsafe { std::mem::transmute(self) };
        let rhs: Simd<u8, 32> = unsafe { std::mem::transmute(rhs) };
        i8x32(lhs.simd_lt(rhs).to_int())
    }
    fn simd_le(self, rhs: Self) -> i8x32 {
        let lhs: Simd<u8, 32> = unsafe { std::mem::transmute(self) };
        let rhs: Simd<u8, 32> = unsafe { std::mem::transmute(rhs) };
        i8x32(lhs.simd_le(rhs).to_int())
    }
    fn simd_gt(self, rhs: Self) -> i8x32 {
        let lhs: Simd<u8, 32> = unsafe { std::mem::transmute(self) };
        let rhs: Simd<u8, 32> = unsafe { std::mem::transmute(rhs) };
        i8x32(lhs.simd_gt(rhs).to_int())
    }
    fn simd_ge(self, rhs: Self) -> i8x32 {
        let lhs: Simd<u8, 32> = unsafe { std::mem::transmute(self) };
        let rhs: Simd<u8, 32> = unsafe { std::mem::transmute(rhs) };
        i8x32(lhs.simd_ge(rhs).to_int())
    }
}

impl std::ops::Add for boolx32 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut ret = boolx32::default();
        for i in 0..32 {
            ret.0[i] = self.0[i] || rhs.0[i];
        }
        ret
    }
}
impl std::ops::Sub for boolx32 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut ret = boolx32::default();
        for i in 0..32 {
            ret.0[i] = self.0[i] && !rhs.0[i];
        }
        ret
    }
}
impl std::ops::Mul for boolx32 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut ret = boolx32::default();
        for i in 0..32 {
            ret.0[i] = self.0[i] && rhs.0[i];
        }
        ret
    }
}
impl std::ops::Div for boolx32 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let mut ret = boolx32::default();
        for i in 0..32 {
            ret.0[i] = self.0[i] && !rhs.0[i];
        }
        ret
    }
}
impl std::ops::Rem for boolx32 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        let mut ret = boolx32::default();
        for i in 0..32 {
            ret.0[i] = self.0[i] ^ rhs.0[i];
        }
        ret
    }
}
impl std::ops::BitOr for boolx32 {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        let mask: Simd<u8, 32> = unsafe { std::mem::transmute(self) };
        let rhs: Simd<u8, 32> = unsafe { std::mem::transmute(rhs) };
        boolx32(unsafe { std::mem::transmute(mask | rhs) })
    }
}
impl std::ops::BitAnd for boolx32 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        let mask: Simd<u8, 32> = unsafe { std::mem::transmute(self) };
        let rhs: Simd<u8, 32> = unsafe { std::mem::transmute(rhs) };
        boolx32(unsafe { std::mem::transmute(mask & rhs) })
    }
}
