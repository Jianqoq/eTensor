use std::borrow::{Borrow, BorrowMut};

use cudarc::{
    cublas::{CudaBlas, Gemm},
    driver::DeviceRepr,
};
use tensor_cudakernels::MATMUL;
use tensor_traits::{CommonBounds, Matmul, TensorInfo};

use crate::ops::cuda::cuda_utils::compute_kernel_launch_config;
use crate::{
    ops::cuda::{cuda_utils::load_ptx_and_get_data, matmul::matmul_with_out},
    tensor::Tensor,
    tensor_base::_Tensor,
    Cuda,
};
use tensor_traits::TensorCreator;
impl<T, const CUDA_DEVICE: usize> Matmul<Tensor<T, Cuda, CUDA_DEVICE>>
    for Tensor<T, Cuda, CUDA_DEVICE>
where
    T: CommonBounds + DeviceRepr,
    CudaBlas: Gemm<T>,
{
    type Output = Tensor<T, Cuda, CUDA_DEVICE>;

    type OutputMeta = T;

    type InplaceOutput = Tensor<T, Cuda, CUDA_DEVICE>;

    fn matmul(&self, rhs: Tensor<T, Cuda, CUDA_DEVICE>) -> anyhow::Result<Self::Output> {
        Ok(matmul_with_out(
            self.inner.as_ref(),
            rhs.inner.as_ref(),
            None::<Self::Output>,
        )?
        .into())
    }
    fn matmul_<U>(&self, rhs: Tensor<T, Cuda, CUDA_DEVICE>, out: U) -> anyhow::Result<Self::Output>
    where
        U: Borrow<Self::InplaceOutput> + BorrowMut<Self::InplaceOutput>,
    {
        let out = out.borrow().inner.as_ref().clone();
        Ok(matmul_with_out(self.inner.as_ref(), rhs.inner.as_ref(), Some(out))?.into())
    }
}
use cudarc::driver::LaunchAsync;
impl<T, const CUDA_DEVICE: usize> Tensor<T, Cuda, CUDA_DEVICE>
where
    T: CommonBounds + DeviceRepr,
    CudaBlas: Gemm<T>,
{
    /// Naive matmul implementation
    pub fn matmul_naive(
        &self,
        rhs: &Tensor<T, Cuda, CUDA_DEVICE>,
    ) -> anyhow::Result<Tensor<T, Cuda, CUDA_DEVICE>> {
        let ret = _Tensor::<T, Cuda, CUDA_DEVICE>::zeros(vec![
            self.inner.layout.shape()[0],
            rhs.inner.layout.shape()[1],
        ])?;
        let m = self.inner.layout.shape()[0] as usize;
        let n = rhs.inner.layout.shape()[1] as usize;
        let k = self.inner.layout.shape()[1] as usize;
        let (kernel, reg_info) = load_ptx_and_get_data(
            "matmul",
            "matmul_naive",
            self.device(),
            self.inner.device_cap(),
            &MATMUL,
        )?;
        let mut cfg =
            compute_kernel_launch_config(self.device(), &reg_info, ret.layout.size() as usize);
        cfg.block_dim = (16, 16, 1);
        cfg.grid_dim = (m.div_ceil(16) as u32, n.div_ceil(16) as u32, 1);
        unsafe {
            kernel.launch(
                cfg,
                (
                    self.inner.cuda_slice(),
                    rhs.inner.cuda_slice(),
                    ret.cuda_slice(),
                    m,
                    n,
                    k,
                    1,
                ),
            )?;
        }
        Ok(ret.into())
    }

    /// Blocked matmul implementation
    pub fn matmul_blocked(
        &self,
        rhs: &Tensor<T, Cuda, CUDA_DEVICE>,
    ) -> anyhow::Result<Tensor<T, Cuda, CUDA_DEVICE>> {
        let ret = _Tensor::<T, Cuda, CUDA_DEVICE>::zeros(vec![
            self.inner.layout.shape()[0],
            rhs.inner.layout.shape()[1],
        ])?;
        let m = self.inner.layout.shape()[0] as usize;
        let n = rhs.inner.layout.shape()[1] as usize;
        let k = self.inner.layout.shape()[1] as usize;
        let (kernel, reg_info) = load_ptx_and_get_data(
            "matmul",
            "matmul_blocked2",
            self.device(),
            self.inner.device_cap(),
            &MATMUL,
        )?;
        let mut cfg =
            compute_kernel_launch_config(self.device(), &reg_info, ret.layout.size() as usize);
        cfg.block_dim = (16, 16, 1);
        cfg.grid_dim = (n.div_ceil(64) as u32, m.div_ceil(64) as u32, 1);
        unsafe {
            kernel.launch(
                cfg,
                (
                    self.inner.cuda_slice(),
                    rhs.inner.cuda_slice(),
                    ret.cuda_slice(),
                    m,
                    n,
                    k,
                    1,
                ),
            )?;
        }
        Ok(ret.into())
    }
}

impl<T, const CUDA_DEVICE: usize> Matmul<&Tensor<T, Cuda, CUDA_DEVICE>>
    for Tensor<T, Cuda, CUDA_DEVICE>
where
    T: CommonBounds + DeviceRepr,
    CudaBlas: Gemm<T>,
{
    type Output = Tensor<T, Cuda, CUDA_DEVICE>;

    type OutputMeta = T;

    type InplaceOutput = Tensor<T, Cuda, CUDA_DEVICE>;

    fn matmul(&self, rhs: &Tensor<T, Cuda, CUDA_DEVICE>) -> anyhow::Result<Self::Output> {
        Ok(matmul_with_out(
            self.inner.as_ref(),
            rhs.inner.as_ref(),
            None::<Self::Output>,
        )?
        .into())
    }

    fn matmul_<U>(&self, rhs: &Tensor<T, Cuda, CUDA_DEVICE>, out: U) -> anyhow::Result<Self::Output>
    where
        U: Borrow<Self::InplaceOutput> + BorrowMut<Self::InplaceOutput>,
    {
        let out = out.borrow().inner.as_ref().clone();
        Ok(matmul_with_out(self.inner.as_ref(), rhs.inner.as_ref(), Some(out))?.into())
    }
}
