use crate::convertion::VecConvertor;
use crate::traits::VecTrait;
use crate::vectors::arch_simd::_256bit::u8x32::u8x32;
use crate::traits::SimdCompare;

use super::i8x32::i8x32;

/// a vector of 16 bool values
#[allow(non_camel_case_types)]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
#[repr(C, align(32))]
pub struct boolx32(pub(crate) [bool; 32]);

impl VecTrait<bool> for boolx32 {
    const SIZE: usize = 32;
    type Base = bool;
    #[inline(always)]
    fn copy_from_slice(&mut self, slice: &[bool]) {
        self.0.copy_from_slice(slice);
    }
    #[inline(always)]
    fn mul_add(self, _: Self, _: Self) -> Self {
        todo!()
    }
    #[inline(always)]
    fn sum(&self) -> bool {
        self.0.iter().map(|&x| x as u8).sum::<u8>() > 0
    }
    fn splat(val: bool) -> boolx32 {
        boolx32([val; 32])
    }
}

impl boolx32 {
    #[allow(unused)]
    fn as_array(&self) -> [bool; 32] {
        unsafe { std::mem::transmute(self.0) }
    }
}

impl SimdCompare for boolx32 {
    type SimdMask = i8x32;
    fn simd_eq(self, rhs: Self) -> i8x32 {
        let mut res = [0i8; 32];
        for i in 0..32 {
            res[i] = if self.0[i] == rhs.0[i] { -1 } else { 0 };
        }
        i8x32(unsafe { std::mem::transmute(res) })
    }
    fn simd_ne(self, rhs: Self) -> i8x32 {
        let mut res = [0i8; 32];
        for i in 0..32 {
            res[i] = if self.0[i] != rhs.0[i] { -1 } else { 0 };
        }
        i8x32(unsafe { std::mem::transmute(res) })
    }
    fn simd_lt(self, rhs: Self) -> i8x32 {
        let mut res = [0i8; 32];
        for i in 0..32 {
            res[i] = if self.0[i] < rhs.0[i] { -1 } else { 0 };
        }
        i8x32(unsafe { std::mem::transmute(res) })
    }
    fn simd_le(self, rhs: Self) -> i8x32 {
        let mut res = [0i8; 32];
        for i in 0..32 {
            res[i] = if self.0[i] <= rhs.0[i] { -1 } else { 0 };
        }
        i8x32(unsafe { std::mem::transmute(res) })
    }
    fn simd_gt(self, rhs: Self) -> i8x32 {
        let mut res = [0i8; 32];
        for i in 0..32 {
            res[i] = if self.0[i] > rhs.0[i] { -1 } else { 0 };
        }
        i8x32(unsafe { std::mem::transmute(res) })
    }
    fn simd_ge(self, rhs: Self) -> i8x32 {
        let mut res = [0i8; 32];
        for i in 0..32 {
            res[i] = if self.0[i] >= rhs.0[i] { -1 } else { 0 };
        }
        i8x32(unsafe { std::mem::transmute(res) })
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
        let mask: u8x32 = unsafe { std::mem::transmute(self) };
        let rhs: u8x32 = unsafe { std::mem::transmute(rhs) };
        boolx32(unsafe { std::mem::transmute(mask | rhs) })
    }
}
impl std::ops::BitAnd for boolx32 {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        let mask: u8x32 = unsafe { std::mem::transmute(self) };
        let rhs: u8x32 = unsafe { std::mem::transmute(rhs) };
        boolx32(unsafe { std::mem::transmute(mask & rhs) })
    }
}

impl VecConvertor for boolx32 {
    fn to_bool(self) -> boolx32 {
        self
    }
    fn to_i8(self) -> i8x32 {
        unsafe { std::mem::transmute(self) }
    }
    fn to_u8(self) -> u8x32 {
        unsafe { std::mem::transmute(self) }
    }
}
