use num_complex::Complex32;

use crate::into_vec::IntoVec;

use crate::vectors::traits::{ Init, VecSize, VecTrait };

#[allow(non_camel_case_types)]
#[derive(Default, Clone, Copy, PartialEq)]
pub struct cplx32x8(pub(crate) [Complex32; 8]);

impl VecTrait<Complex32> for cplx32x8 {
    #[inline(always)]
    fn copy_from_slice(&mut self, slice: &[Complex32]) {
        self.0.copy_from_slice(slice);
    }
    #[inline(always)]
    fn as_ptr(&self) -> *const Complex32 {
        self.0.as_ptr()
    }
    #[inline(always)]
    fn _mul_add(self, _: Self, _: Self) -> Self {
        todo!()
    }
    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut Complex32 {
        self.0.as_mut_ptr()
    }
    #[inline(always)]
    fn as_mut_ptr_uncheck(&self) -> *mut Complex32 {
        self.0.as_ptr() as *mut _
    }
    #[inline(always)]
    fn sum(&self) -> Complex32 {
        self.0.iter().sum()
    }

    fn extract(self, idx: usize) -> Complex32 {
        self.0[idx]
    }
}
impl VecSize for cplx32x8 {
    const SIZE: usize = 8;
}
impl Init<Complex32> for cplx32x8 {
    fn splat(val: Complex32) -> cplx32x8 {
        cplx32x8([val; 8])
    }

    unsafe fn from_ptr(ptr: *const Complex32) -> Self {
        let mut tmp = core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            core::ptr::copy_nonoverlapping(ptr, tmp.as_mut_ptr().cast(), 1);
            tmp.assume_init()
        }
    }
}
impl IntoVec<cplx32x8> for cplx32x8 {
    fn into_vec(self) -> cplx32x8 {
        self
    }
}
impl std::ops::Add for cplx32x8 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut ret = cplx32x8::default();
        for i in 0..8 {
            ret.0[i] = self.0[i] + rhs.0[i];
        }
        ret
    }
}
impl std::ops::Sub for cplx32x8 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut ret = cplx32x8::default();
        for i in 0..8 {
            ret.0[i] = self.0[i] - rhs.0[i];
        }
        ret
    }
}
impl std::ops::Mul for cplx32x8 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut ret = cplx32x8::default();
        for i in 0..8 {
            ret.0[i] = self.0[i] * rhs.0[i];
        }
        ret
    }
}
impl std::ops::Div for cplx32x8 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let mut ret = cplx32x8::default();
        for i in 0..8 {
            ret.0[i] = self.0[i] / rhs.0[i];
        }
        ret
    }
}
