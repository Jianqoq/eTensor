use crate::arch_simd::_128bit::u16x8::u16x8;
use crate::convertion::VecConvertor;
use crate::traits::SimdCompare;
use crate::{traits::VecTrait, vectors::arch_simd::_128bit::f32x4::f32x4};

use super::i16x8::i16x8;

use std::arch::x86_64::*;

/// a vector of 8 bf16 values
#[allow(non_camel_case_types)]
#[derive(Default, Clone, Copy, PartialEq, Debug)]
#[repr(C, align(16))]
pub struct bf16x8(pub(crate) [half::bf16; 8]);

impl VecTrait<half::bf16> for bf16x8 {
    const SIZE: usize = 8;
    type Base = half::bf16;
    #[inline(always)]
    fn copy_from_slice(&mut self, slice: &[half::bf16]) {
        self.0.copy_from_slice(slice);
    }
    #[inline(always)]
    fn mul_add(self, a: Self, b: Self) -> Self {
        let [x0, x1]: [f32x4; 2] = unsafe { std::mem::transmute(self.to_2_f32x4()) };
        let [a0, a1]: [f32x4; 2] = unsafe { std::mem::transmute(a.to_2_f32x4()) };
        let [b0, b1]: [f32x4; 2] = unsafe { std::mem::transmute(b.to_2_f32x4()) };
        let res0 = x0.mul_add(a0, b0);
        let res1 = x1.mul_add(a1, b1);
        bf16x8::from_2_f32x4([res0, res1])
    }
    #[inline(always)]
    fn sum(&self) -> half::bf16 {
        self.0.iter().sum()
    }
    fn splat(val: half::bf16) -> bf16x8 {
        bf16x8([val; 8])
    }
}

impl bf16x8 {
    #[allow(unused)]
    fn as_array(&self) -> [half::bf16; 8] {
        unsafe { std::mem::transmute(self.0) }
    }
}

impl bf16x8 {
    /// convert to 2 f32x4
    pub fn to_2_f32x4(&self) -> [f32x4; 2] {
        todo!()
    }

    /// convert from 2 f32x4
    pub fn from_2_f32x4(_: [f32x4; 2]) -> Self {
        todo!()
    }

    /// check if the value is NaN and return a mask
    pub fn is_nan(&self) -> i16x8 {
        let res: [i16; 8] = self.0.map(|x| if x.is_nan() { 1 } else { 0 });
        unsafe { std::mem::transmute(res) }
    }

    /// check if the value is infinite and return a mask
    pub fn is_infinite(&self) -> u16x8 {
        let x = u16x8::splat(0x7f80u16);
        let y = u16x8::splat(0x007fu16);
        let i: u16x8 = unsafe { std::mem::transmute(self.0) };

        let and = i & x;
        let eq = and.simd_eq(x);

        let and2 = i & y;
        let eq_zero = and2.simd_eq(u16x8::splat(0));

        let result = eq & eq_zero;

        unsafe { std::mem::transmute(result) }
    }
}
impl SimdCompare for bf16x8 {
    type SimdMask = i16x8;
    fn simd_eq(self, other: Self) -> i16x8 {
        unsafe {
            let self_ptr = &self.0 as *const _ as *const __m128i;
            let other_ptr = &other.0 as *const _ as *const __m128i;
            let a = _mm_loadu_si128(self_ptr);
            let b = _mm_loadu_si128(other_ptr);
            i16x8(_mm_cmpeq_epi16(a, b))
        }
    }

    fn simd_ne(self, other: Self) -> i16x8 {
        unsafe {
            let self_ptr = &self.0 as *const _ as *const __m128i;
            let other_ptr = &other.0 as *const _ as *const __m128i;
            let a = _mm_loadu_si128(self_ptr);
            let b = _mm_loadu_si128(other_ptr);
            let eq = _mm_cmpeq_epi16(a, b);
            i16x8(_mm_xor_si128(eq, _mm_set1_epi16(-1)))
        }
    }

    fn simd_lt(self, other: Self) -> i16x8 {
        unsafe {
            let self_ptr = &self.0 as *const _ as *const __m128i;
            let other_ptr = &other.0 as *const _ as *const __m128i;
            let a = _mm_loadu_si128(self_ptr);
            let b = _mm_loadu_si128(other_ptr);
            i16x8(_mm_cmplt_epi16(a, b))
        }
    }
    fn simd_le(self, other: Self) -> i16x8 {
        unsafe {
            let self_ptr = &self.0 as *const _ as *const __m128i;
            let other_ptr = &other.0 as *const _ as *const __m128i;
            let a = _mm_loadu_si128(self_ptr);
            let b = _mm_loadu_si128(other_ptr);
            let lt = _mm_cmplt_epi16(a, b);
            let eq = _mm_cmpeq_epi16(a, b);
            i16x8(_mm_or_si128(lt, eq))
        }
    }
    fn simd_gt(self, other: Self) -> i16x8 {
        unsafe {
            let self_ptr = &self.0 as *const _ as *const __m128i;
            let other_ptr = &other.0 as *const _ as *const __m128i;
            let a = _mm_loadu_si128(self_ptr);
            let b = _mm_loadu_si128(other_ptr);
            i16x8(_mm_cmpgt_epi16(a, b))
        }
    }
    fn simd_ge(self, other: Self) -> i16x8 {
        unsafe {
            let self_ptr = &self.0 as *const _ as *const __m128i;
            let other_ptr = &other.0 as *const _ as *const __m128i;
            let a = _mm_loadu_si128(self_ptr);
            let b = _mm_loadu_si128(other_ptr);
            let gt = _mm_cmpgt_epi16(a, b);
            let eq = _mm_cmpeq_epi16(a, b);
            i16x8(_mm_or_si128(gt, eq))
        }
    }
}

impl std::ops::Add for bf16x8 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut ret = bf16x8::default();
        for i in 0..8 {
            ret.0[i] = self.0[i] + rhs.0[i];
        }
        ret
    }
}
impl std::ops::Sub for bf16x8 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut ret = bf16x8::default();
        for i in 0..8 {
            ret.0[i] = self.0[i] - rhs.0[i];
        }
        ret
    }
}
impl std::ops::Mul for bf16x8 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut ret = bf16x8::default();
        for i in 0..8 {
            ret.0[i] = self.0[i] * rhs.0[i];
        }
        ret
    }
}
impl std::ops::Div for bf16x8 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let mut ret = bf16x8::default();
        for i in 0..8 {
            ret.0[i] = self.0[i] / rhs.0[i];
        }
        ret
    }
}
impl std::ops::Rem for bf16x8 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        let mut ret = bf16x8::default();
        for i in 0..8 {
            ret.0[i] = self.0[i] % rhs.0[i];
        }
        ret
    }
}
impl std::ops::Neg for bf16x8 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let mut ret = bf16x8::default();
        for i in 0..8 {
            ret.0[i] = -self.0[i];
        }
        ret
    }
}

impl VecConvertor for bf16x8 {
    fn to_bf16(self) -> bf16x8 {
        self
    }
    fn to_f16(self) -> super::f16x8::f16x8 {
        unsafe { std::mem::transmute(self) }
    }
    fn to_i16(self) -> super::i16x8::i16x8 {
        unsafe {
            let [x0, x1]: [f32x4; 2] = std::mem::transmute(self.to_2_f32x4());
            let i0 = _mm_cvtps_epi32(x0.0);
            let i1 = _mm_cvtps_epi32(x1.0);
            let packed = _mm_packs_epi32(i0, i1);
            super::i16x8::i16x8(packed)
        }
    }
    fn to_u16(self) -> super::u16x8::u16x8 {
        unsafe {
            let [x0, x1]: [f32x4; 2] = std::mem::transmute(self.to_2_f32x4());
            let i0 = _mm_cvtps_epi32(x0.0);
            let i1 = _mm_cvtps_epi32(x1.0);
            let packed = _mm_packus_epi32(i0, i1);
            super::u16x8::u16x8(packed)
        }
    }
}
