use crate::{convertion::VecConvertor, traits::{ SimdCompare, SimdMath, SimdSelect, VecTrait }};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

use super::i64x2::i64x2;

/// a vector of 2 u64 values
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(C, align(16))]
pub struct u64x2(
    #[cfg(target_arch = "x86_64")] pub(crate) __m128i,
    #[cfg(target_arch = "aarch64")] pub(crate) uint64x2_t);

impl PartialEq for u64x2 {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let cmp = _mm_cmpeq_epi64(self.0, other.0);
            _mm_movemask_epi8(cmp) == -1
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            let cmp = vceqq_u64(self.0, other.0);
            vgetq_lane_u64(cmp, 0) == 0xFFFFFFFFFFFFFFFF && vgetq_lane_u64(cmp, 1) == 0xFFFFFFFFFFFFFFFF
        }
    }
}

impl Default for u64x2 {
    #[inline(always)]
    fn default() -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe { u64x2(_mm_setzero_si128()) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u64x2(vdupq_n_u64(0)) }
    }
}

impl VecTrait<u64> for u64x2 {
    const SIZE: usize = 2;
    type Base = u64;
    #[inline(always)]
    fn copy_from_slice(&mut self, slice: &[u64]) {
        #[cfg(target_arch = "x86_64")]
        unsafe { _mm_storeu_si128(&mut self.0, _mm_loadu_si128(slice.as_ptr() as *const __m128i)) }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            self.0 = vld1q_u64(slice.as_ptr());
        }
    }
    #[inline(always)]
    fn mul_add(self, a: Self, b: Self) -> Self {
        unsafe {
            let arr: [u64; 2] = std::mem::transmute(self.0);
            let arr2: [u64; 2] = std::mem::transmute(a.0);
            let arr3: [u64; 2] = std::mem::transmute(b.0);
            let mut arr4: [u64; 2] = [0; 2];
            for i in 0..2 {
                arr4[i] = arr[i] * arr2[i] + arr3[i];
            }
            #[cfg(target_arch = "x86_64")]
            return u64x2(_mm_loadu_si128(arr4.as_ptr() as *const __m128i));
            #[cfg(target_arch = "aarch64")]
            return u64x2(vld1q_u64(arr4.as_ptr()));
        }
    }
    #[inline(always)]
    fn sum(&self) -> u64 {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let arr: [u64; 2] = std::mem::transmute(self.0);
            arr.iter().sum()
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            vaddvq_u64(self.0)
        }
    }
    #[inline(always)]
    fn splat(val: u64) -> u64x2 {
        #[cfg(target_arch = "x86_64")]
        unsafe { u64x2(_mm_set1_epi64x(val as i64)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u64x2(vdupq_n_u64(val)) }
    }
}

impl u64x2 {
    #[allow(unused)]
    #[inline(always)]
    fn as_array(&self) -> [u64; 2] {
        unsafe { std::mem::transmute(self.0) }
    }
}


impl SimdCompare for u64x2 {
    type SimdMask = i64x2;

    #[inline(always)]
    fn simd_eq(self, other: Self) -> Self::SimdMask {
        unsafe {
            let lhs: i64x2 = std::mem::transmute(self.0);
            let rhs: i64x2 = std::mem::transmute(other.0);
            lhs.simd_eq(rhs)
        }
    }

    #[inline(always)]
    fn simd_ne(self, other: Self) -> Self::SimdMask {
        unsafe {
            let lhs: i64x2 = std::mem::transmute(self.0);
            let rhs: i64x2 = std::mem::transmute(other.0);
            lhs.simd_ne(rhs)
        }
    }

    #[inline(always)]
    fn simd_lt(self, other: Self) -> Self::SimdMask {
        unsafe {
            let lhs: i64x2 = std::mem::transmute(self.0);
            let rhs: i64x2 = std::mem::transmute(other.0);
            lhs.simd_lt(rhs)
        }
    }

    #[inline(always)]
    fn simd_le(self, other: Self) -> Self::SimdMask {
        unsafe {
            let lhs: i64x2 = std::mem::transmute(self.0);
            let rhs: i64x2 = std::mem::transmute(other.0);
            lhs.simd_le(rhs)
        }
    }

    #[inline(always)]
    fn simd_gt(self, other: Self) -> Self::SimdMask {
        unsafe {
            let lhs: i64x2 = std::mem::transmute(self.0);
            let rhs: i64x2 = std::mem::transmute(other.0);
            lhs.simd_gt(rhs)
        }
    }

    #[inline(always)]
    fn simd_ge(self, other: Self) -> Self::SimdMask {
        unsafe {
            let lhs: i64x2 = std::mem::transmute(self.0);
            let rhs: i64x2 = std::mem::transmute(other.0);
            lhs.simd_ge(rhs)
        }
    }
}

impl SimdSelect<u64x2> for u64x2 {
    #[inline(always)]
    fn select(&self, true_val: u64x2, false_val: u64x2) -> u64x2 {
        #[cfg(target_arch = "x86_64")]
        unsafe { u64x2(_mm_blendv_epi8(false_val.0, true_val.0, self.0)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u64x2(vbslq_u64(self.0, true_val.0, false_val.0)) }
    }
}

impl std::ops::Add for u64x2 {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe { u64x2(_mm_add_epi64(self.0, rhs.0)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u64x2(vaddq_u64(self.0, rhs.0)) }
    }
}
impl std::ops::Sub for u64x2 {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe { u64x2(_mm_sub_epi64(self.0, rhs.0)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u64x2(vsubq_u64(self.0, rhs.0)) }
    }
}
impl std::ops::Mul for u64x2 {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        unsafe {
            let arr: [u64; 2] = std::mem::transmute(self.0);
            let arr2: [u64; 2] = std::mem::transmute(rhs.0);
            let mut arr3: [u64; 2] = [0; 2];
            for i in 0..2 {
                arr3[i] = arr[i] * arr2[i];
            }
            #[cfg(target_arch = "x86_64")]
            return u64x2(_mm_loadu_si128(arr3.as_ptr() as *const __m128i));
            #[cfg(target_arch = "aarch64")]
            return u64x2(vld1q_u64(arr3.as_ptr()));
        }
    }
}
impl std::ops::Div for u64x2 {
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: Self) -> Self {
        unsafe {
            let arr: [u64; 2] = std::mem::transmute(self.0);
            let arr2: [u64; 2] = std::mem::transmute(rhs.0);
            let mut arr3: [u64; 2] = [0; 2];
            for i in 0..2 {
                arr3[i] = arr[i] / arr2[i];
            }
            #[cfg(target_arch = "x86_64")]
            return u64x2(_mm_loadu_si128(arr3.as_ptr() as *const __m128i));
            #[cfg(target_arch = "aarch64")]
            return u64x2(vld1q_u64(arr3.as_ptr()));
        }
    }
}
impl std::ops::Rem for u64x2 {
    type Output = Self;
    #[inline(always)]
    fn rem(self, rhs: Self) -> Self {
        unsafe {
            let arr: [u64; 2] = std::mem::transmute(self.0);
            let arr2: [u64; 2] = std::mem::transmute(rhs.0);
            let mut arr3: [u64; 2] = [0; 2];
            for i in 0..2 {
                arr3[i] = arr[i] % arr2[i];
            }
            #[cfg(target_arch = "x86_64")]
            return u64x2(_mm_loadu_si128(arr3.as_ptr() as *const __m128i));
            #[cfg(target_arch = "aarch64")]
            return u64x2(vld1q_u64(arr3.as_ptr()));
        }
    }
}
impl std::ops::BitAnd for u64x2 {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe { u64x2(_mm_and_si128(self.0, rhs.0)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u64x2(vandq_u64(self.0, rhs.0)) }
    }
}
impl std::ops::BitOr for u64x2 {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe { u64x2(_mm_or_si128(self.0, rhs.0)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u64x2(vorrq_u64(self.0, rhs.0)) }
    }
}
impl std::ops::BitXor for u64x2 {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe { u64x2(_mm_xor_si128(self.0, rhs.0)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u64x2(veorq_u64(self.0, rhs.0)) }
    }
}
impl std::ops::Not for u64x2 {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe { u64x2(_mm_xor_si128(self.0, _mm_set1_epi64x(-1))) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u64x2(veorq_u64(self.0, vdupq_n_u64(0xFFFFFFFFFFFFFFFF))) }
    }
}
impl std::ops::Shl for u64x2 {
    type Output = Self;
    #[inline(always)]
    fn shl(self, rhs: Self) -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let a: [u64; 2] = std::mem::transmute(self.0);
            let b: [u64; 2] = std::mem::transmute(rhs.0);
            let mut result = [0; 2];
            for i in 0..2 {
                result[i] = a[i].wrapping_shl(b[i] as u32);
            }
            u64x2(_mm_loadu_si128(result.as_ptr() as *const __m128i))
        }
        #[cfg(target_arch = "aarch64")]
        unsafe { u64x2(vshlq_u64(self.0, vreinterpretq_s64_u64(rhs.0))) }
    }
}
impl std::ops::Shr for u64x2 {
    type Output = Self;
    #[inline(always)]
    fn shr(self, rhs: Self) -> Self {
        unsafe {
            let a: [u64; 2] = std::mem::transmute(self.0);
            let b: [u64; 2] = std::mem::transmute(rhs.0);
            let mut result = [0; 2];
            for i in 0..2 {
                result[i] = a[i].wrapping_shr(b[i] as u32);
            }
            #[cfg(target_arch = "x86_64")]
            return u64x2(_mm_loadu_si128(result.as_ptr() as *const __m128i));
            #[cfg(target_arch = "aarch64")]
            return u64x2(vld1q_u64(result.as_ptr()));
        }
    }
}
impl SimdMath<u64> for u64x2 {
    #[inline(always)]
    fn max(self, other: Self) -> Self {
        unsafe {
            let arr: [u64; 2] = std::mem::transmute(self.0);
            let arr2: [u64; 2] = std::mem::transmute(other.0);
            let mut arr3: [u64; 2] = [0; 2];
            for i in 0..2 {
                arr3[i] = arr[i].max(arr2[i]);
            }
            #[cfg(target_arch = "x86_64")]
            return u64x2(_mm_loadu_si128(arr3.as_ptr() as *const __m128i));
            #[cfg(target_arch = "aarch64")]
            return u64x2(vld1q_u64(arr3.as_ptr()));
        }
    }
    #[inline(always)]
    fn min(self, other: Self) -> Self {
        unsafe {
            let arr: [u64; 2] = std::mem::transmute(self.0);
            let arr2: [u64; 2] = std::mem::transmute(other.0);
            let mut arr3: [u64; 2] = [0; 2];
            for i in 0..2 {
                arr3[i] = arr[i].min(arr2[i]);
            }
            #[cfg(target_arch = "x86_64")]
            return u64x2(_mm_loadu_si128(arr3.as_ptr() as *const __m128i));
            #[cfg(target_arch = "aarch64")]
            return u64x2(vld1q_u64(arr3.as_ptr()));
        }
    }
    #[inline(always)]
    fn relu(self) -> Self {
        unsafe {
            let arr: [u64; 2] = std::mem::transmute(self.0);
            let mut arr2: [u64; 2] = [0; 2];
            for i in 0..2 {
                arr2[i] = arr[i].max(0);
            }
            #[cfg(target_arch = "x86_64")]
            return u64x2(_mm_loadu_si128(arr2.as_ptr() as *const __m128i));
            #[cfg(target_arch = "aarch64")]
            return u64x2(vld1q_u64(arr2.as_ptr()));
        }
    }
    #[inline(always)]
    fn relu6(self) -> Self {
        unsafe {
            let arr: [u64; 2] = std::mem::transmute(self.0);
            let mut arr2: [u64; 2] = [0; 2];
            for i in 0..2 {
                arr2[i] = arr[i].max(0).min(6);
            }
            #[cfg(target_arch = "x86_64")]
            return u64x2(_mm_loadu_si128(arr2.as_ptr() as *const __m128i));
            #[cfg(target_arch = "aarch64")]
            return u64x2(vld1q_u64(arr2.as_ptr()));
        }
    }
}

impl VecConvertor for u64x2 {
    #[inline(always)]
    fn to_u64(self) -> u64x2 {
        self
    }
    #[inline(always)]
    fn to_i64(self) -> i64x2 {
        unsafe { std::mem::transmute(self) }
    }
    #[inline(always)]
    fn to_f64(self) -> super::f64x2::f64x2 {
        unsafe {
            let arr: [u64; 2] = std::mem::transmute(self.0);
            let mut result = [0.0f64; 2];
            for i in 0..2 {
                result[i] = arr[i] as f64;
            }
            #[cfg(target_arch = "x86_64")]
            return super::f64x2::f64x2(_mm_loadu_pd(result.as_ptr()));
            #[cfg(target_arch = "aarch64")]
            return super::f64x2::f64x2(vld1q_f64(result.as_ptr()));
        }
    }
    #[inline(always)]
    #[cfg(target_pointer_width = "64")]
    fn to_isize(self) -> super::isizex2::isizex2 {
        unsafe { std::mem::transmute(self) }
    }
    #[inline(always)]
    #[cfg(target_pointer_width = "64")]
    fn to_usize(self) -> super::usizex2::usizex2 {
        unsafe { std::mem::transmute(self) }
    }
}