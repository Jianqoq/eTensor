use crate::{convertion::VecConvertor, traits::{ SimdCompare, SimdMath, SimdSelect, VecTrait }};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

use super::i32x4::i32x4;

/// a vector of 4 u32 values
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(C, align(16))]
pub struct u32x4(
    #[cfg(target_arch = "x86_64")]
    pub(crate) __m128i,
    #[cfg(target_arch = "aarch64")]
    pub(crate) uint32x4_t,
);

impl Default for u32x4 {
    #[inline(always)]
    fn default() -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe { u32x4(_mm_setzero_si128()) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(vdupq_n_u32(0)) }
    }
}

impl PartialEq for u32x4 {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let cmp = _mm_cmpeq_epi32(self.0, other.0);
            _mm_movemask_epi8(cmp) == -1
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            let cmp = vceqq_u32(self.0, other.0);
            vmaxvq_u32(cmp) == 0xFFFFFFFF && vminvq_u32(cmp) == 0xFFFFFFFF
        }
    }
}
impl VecTrait<u32> for u32x4 {
    const SIZE: usize = 4;
    type Base = u32;
    #[inline(always)]
    fn copy_from_slice(&mut self, slice: &[u32]) {
        #[cfg(target_arch = "x86_64")]
        unsafe { _mm_storeu_si128(&mut self.0, _mm_loadu_si128(slice.as_ptr() as *const __m128i)) }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            self.0 = vld1q_u32(slice.as_ptr());
        }
    }
    #[inline(always)]
    fn mul_add(self, a: Self, b: Self) -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe { u32x4(_mm_add_epi32(self.0, _mm_mullo_epi32(a.0, b.0))) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(vaddq_u32(self.0, vmulq_u32(a.0, b.0))) }
    }
    #[inline(always)]
    fn sum(&self) -> u32 {
        unsafe {
            let arr: [u32; 4] = std::mem::transmute(self.0);
            arr.iter().sum()
        }
    }
    #[inline(always)]
    fn splat(val: u32) -> u32x4 {
        #[cfg(target_arch = "x86_64")]
        unsafe { u32x4(_mm_set1_epi32(val as i32)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(vdupq_n_u32(val)) }
    }
}

impl u32x4 {
    #[allow(unused)]
    #[inline(always)]
    fn as_array(&self) -> [u32; 4] {
        unsafe { std::mem::transmute(self.0) }
    }
}

impl SimdCompare for u32x4 {
    type SimdMask = i32x4;

    #[inline(always)]
    fn simd_eq(self, other: Self) -> Self::SimdMask {
        unsafe {
            let lhs: i32x4 = std::mem::transmute(self.0);
            let rhs: i32x4 = std::mem::transmute(other.0);
            lhs.simd_eq(rhs)
        }
    }

    #[inline(always)]
    fn simd_ne(self, other: Self) -> Self::SimdMask {
        unsafe {
            let lhs: i32x4 = std::mem::transmute(self.0);
            let rhs: i32x4 = std::mem::transmute(other.0);
            lhs.simd_ne(rhs)
        }
    }

    #[inline(always)]
    fn simd_lt(self, other: Self) -> Self::SimdMask {
        unsafe {
            let lhs: i32x4 = std::mem::transmute(self.0);
            let rhs: i32x4 = std::mem::transmute(other.0);
            lhs.simd_lt(rhs)
        }
    }

    #[inline(always)]
    fn simd_le(self, other: Self) -> Self::SimdMask {
        unsafe {
            let lhs: i32x4 = std::mem::transmute(self.0);
            let rhs: i32x4 = std::mem::transmute(other.0);
            lhs.simd_le(rhs)
        }
    }

    #[inline(always)]
    fn simd_gt(self, other: Self) -> Self::SimdMask {
        unsafe {
            let lhs: i32x4 = std::mem::transmute(self.0);
            let rhs: i32x4 = std::mem::transmute(other.0);
            lhs.simd_gt(rhs)
        }
    }

    #[inline(always)]
    fn simd_ge(self, other: Self) -> Self::SimdMask {
        unsafe {
            let lhs: i32x4 = std::mem::transmute(self.0);
            let rhs: i32x4 = std::mem::transmute(other.0);
            lhs.simd_ge(rhs)
        }
    }
}

impl SimdSelect<u32x4> for u32x4 {
    #[inline(always)]
    fn select(&self, true_val: u32x4, false_val: u32x4) -> u32x4 {
        #[cfg(target_arch = "x86_64")]
        unsafe { u32x4(_mm_blendv_epi8(false_val.0, true_val.0, self.0)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(vbslq_u32(self.0, true_val.0, false_val.0)) }
    }
}

impl std::ops::Add for u32x4 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        #[cfg(target_arch = "x86_64")]
        unsafe { u32x4(_mm_add_epi32(self.0, rhs.0)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(vaddq_u32(self.0, rhs.0)) }
    }
}
impl std::ops::Sub for u32x4 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        #[cfg(target_arch = "x86_64")]
        unsafe { u32x4(_mm_sub_epi32(self.0, rhs.0)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(vsubq_u32(self.0, rhs.0)) }
    }
}
impl std::ops::Mul for u32x4 {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        #[cfg(target_arch = "x86_64")]
        unsafe { u32x4(_mm_mullo_epi32(self.0, rhs.0)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(vmulq_u32(self.0, rhs.0)) }
    }
}
impl std::ops::Div for u32x4 {
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        unsafe {
            let arr: [u32; 4] = std::mem::transmute(self.0);
            let arr2: [u32; 4] = std::mem::transmute(rhs.0);
            let mut arr3: [u32; 4] = [0; 4];
            for i in 0..4 {
                arr3[i] = arr[i] / arr2[i];
            }
            #[cfg(target_arch = "x86_64")]
            return u32x4(_mm_loadu_si128(arr3.as_ptr() as *const __m128i));
            #[cfg(target_arch = "aarch64")]
            return u32x4(vld1q_u32(arr3.as_ptr()));
        }
    }
}
impl std::ops::Rem for u32x4 {
    type Output = Self;
    #[inline(always)]
    fn rem(self, rhs: Self) -> Self::Output {
        unsafe {
            let arr: [u32; 4] = std::mem::transmute(self.0);
            let arr2: [u32; 4] = std::mem::transmute(rhs.0);
            let mut arr3: [u32; 4] = [0; 4];
            for i in 0..4 {
                arr3[i] = arr[i] % arr2[i];
            }
            #[cfg(target_arch = "x86_64")]
            return u32x4(_mm_loadu_si128(arr3.as_ptr() as *const __m128i));
            #[cfg(target_arch = "aarch64")]
            return u32x4(vld1q_u32(arr3.as_ptr()));
        }
    }
}
impl std::ops::BitAnd for u32x4 {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        #[cfg(target_arch = "x86_64")]
        unsafe { u32x4(_mm_and_si128(self.0, rhs.0)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(vandq_u32(self.0, rhs.0)) }
    }
}
impl std::ops::BitOr for u32x4 {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        #[cfg(target_arch = "x86_64")]
        unsafe { u32x4(_mm_or_si128(self.0, rhs.0)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(vorrq_u32(self.0, rhs.0)) }
    }
}
impl std::ops::BitXor for u32x4 {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        #[cfg(target_arch = "x86_64")]
        unsafe { u32x4(_mm_xor_si128(self.0, rhs.0)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(veorq_u32(self.0, rhs.0)) }
    }
}
impl std::ops::Not for u32x4 {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self::Output {
        #[cfg(target_arch = "x86_64")]
        unsafe { u32x4(_mm_xor_si128(self.0, _mm_set1_epi32(-1))) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(vmvnq_u32(self.0)) }
    }
}
impl std::ops::Shl for u32x4 {
    type Output = Self;
    #[inline(always)]
    fn shl(self, rhs: Self) -> Self::Output {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let a: [u32; 4] = std::mem::transmute(self.0);
            let b: [u32; 4] = std::mem::transmute(rhs.0);
            let mut result = [0; 4];
            for i in 0..4 {
                result[i] = a[i].wrapping_shl(b[i] as u32);
            }
            u32x4(_mm_loadu_si128(result.as_ptr() as *const __m128i))
        }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(vshlq_u32(self.0, vreinterpretq_s32_u32(rhs.0))) }
    }
}
impl std::ops::Shr for u32x4 {
    type Output = Self;
    #[inline(always)]
    fn shr(self, rhs: Self) -> Self::Output {
        unsafe {
            let a: [u32; 4] = std::mem::transmute(self.0);
            let b: [u32; 4] = std::mem::transmute(rhs.0);
            let mut result = [0; 4];
            for i in 0..4 {
                result[i] = a[i].wrapping_shr(b[i] as u32);
            }
            #[cfg(target_arch = "x86_64")]
            return u32x4(_mm_loadu_si128(result.as_ptr() as *const __m128i));
            #[cfg(target_arch = "aarch64")]
            return u32x4(vld1q_u32(result.as_ptr()));
        }
    }
}

impl SimdMath<u32> for u32x4 {
    #[inline(always)]
    fn max(self, other: Self) -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe { u32x4(_mm_max_epi32(self.0, other.0)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(vmaxq_u32(self.0, other.0)) }
    }
    #[inline(always)]
    fn min(self, other: Self) -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe { u32x4(_mm_min_epi32(self.0, other.0)) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(vminq_u32(self.0, other.0)) }
    }
    #[inline(always)]
    fn relu(self) -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe { u32x4(_mm_max_epi32(self.0, _mm_setzero_si128())) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(vmaxq_u32(self.0, vdupq_n_u32(0))) }
    }
    #[inline(always)]
    fn relu6(self) -> Self {
        #[cfg(target_arch = "x86_64")]
        unsafe { u32x4(_mm_min_epi32(self.relu().0, _mm_set1_epi32(6))) }
        #[cfg(target_arch = "aarch64")]
        unsafe { u32x4(vminq_u32(self.relu().0, vdupq_n_u32(6))) }
    }
}

impl VecConvertor for u32x4 {
    #[inline(always)]
    fn to_u32(self) -> u32x4 {
        self
    }
    #[inline(always)]
    fn to_i32(self) -> i32x4 {
        unsafe { std::mem::transmute(self) }
    }
    #[inline(always)]
    fn to_f32(self) -> super::f32x4::f32x4 {
        unsafe {
            let arr: [u32; 4] = std::mem::transmute(self.0);
            let mut result = [0.0f32; 4];
            for i in 0..4 {
                result[i] = arr[i] as f32;
            }
            #[cfg(target_arch = "x86_64")]
            return super::f32x4::f32x4(_mm_loadu_ps(result.as_ptr()));
            #[cfg(target_arch = "aarch64")]
            return super::f32x4::f32x4(vld1q_f32(result.as_ptr()));
        }
    }
    #[inline(always)]
    #[cfg(target_pointer_width = "32")]
    fn to_usize(self) -> super::usizex2::usizex2 {
        unsafe { std::mem::transmute(self) }
    }
    #[inline(always)]
    #[cfg(target_pointer_width = "32")]
    fn to_isize(self) -> super::isizex2::isizex2 {
        unsafe { std::mem::transmute(self) }
    }
}

impl FloatOutBinary2 for u32x4 {
    #[inline(always)]
    fn __div(self, rhs: Self) -> Self {
        self / rhs
    }

    #[inline(always)]
    fn __log(self, _: Self) -> Self {
        panic!("Logarithm operation is not supported for u16")
    }
}

impl NormalOut2 for u32x4 {
    #[inline(always)]
    fn __add(self, rhs: Self) -> Self {
        self + rhs
    }

    #[inline(always)]
    fn __sub(self, rhs: Self) -> Self {
        self - rhs
    }

    #[inline(always)]
    fn __mul_add(self, a: Self, b: Self) -> Self {
        self.mul_add(a, b)
    }

    #[inline(always)]
    fn __mul(self, rhs: Self) -> Self {
        self * rhs
    }

    #[inline(always)]
    fn __pow(self, rhs: Self) -> Self {
        self.pow(rhs)
    }

    #[inline(always)]
    fn __rem(self, rhs: Self) -> Self {
        self % rhs
    }

    #[inline(always)]
    fn __max(self, rhs: Self) -> Self {
        self.max(rhs)
    }

    #[inline(always)]
    fn __min(self, rhs: Self) -> Self {
        self.min(rhs)
    }

    #[inline(always)]
    fn __clip(self, min: Self, max: Self) -> Self {
        self.max(min).min(max)
    }
}

impl NormalOutUnary2 for u32x4 {
    #[inline(always)]
    fn __square(self) -> Self {
        self * self
    }

    #[inline(always)]
    fn __abs(self) -> Self {
        self
    }

    #[inline(always)]
    fn __ceil(self) -> Self {
        self
    }

    #[inline(always)]
    fn __floor(self) -> Self {
        self
    }

    #[inline(always)]
    fn __neg(self) -> Self {
        self
    }

    #[inline(always)]
    fn __round(self) -> Self {
        self
    }

    #[inline(always)]
    fn __sign(self) -> Self {
        self.sign()
    }

    #[inline(always)]
    fn __leaky_relu(self, alpha: Self) -> Self {
        self.max(u32x4::splat(0)) + alpha * self.min(u32x4::splat(0))
    }

    #[inline(always)]
    fn __relu(self) -> Self {
        self.relu()
    }

    #[inline(always)]
    fn __relu6(self) -> Self {
        self.relu6()
    }
}

impl Eval2 for u32x4 {
    type Output = i32x4;
    #[inline(always)]
    fn __is_nan(&self) -> Self::Output {
        i32x4::default()
    }

    #[inline(always)]
    fn __is_true(&self) -> Self::Output {
        unsafe {
            let eq = _mm_cmpeq_epi32(self.0, _mm_setzero_si128());
            i32x4(_mm_xor_si128(eq, _mm_set1_epi32(-1)))
        }
    }

    #[inline(always)]
    fn __is_inf(&self) -> Self::Output {
        i32x4::default()
    }
}
