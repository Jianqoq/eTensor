use std::borrow::{ Borrow, BorrowMut };

use tensor_traits::{ CommonBounds, Matmul };
use tensor_types::{ into_scalar::IntoScalar, type_promote::NormalOut };

use crate::{ ops::cpu::matmul::matmul_with_out, tensor::Tensor };

impl<A, B> Matmul<Tensor<B>>
    for Tensor<A>
    where
        A: CommonBounds + NormalOut<B> + IntoScalar<<A as NormalOut<B>>::Output>,
        B: CommonBounds + IntoScalar<<A as NormalOut<B>>::Output>,
        <A as NormalOut<B>>::Output: CommonBounds
{
    type Output = Tensor<<A as NormalOut<B>>::Output>;

    type OutputMeta = <A as NormalOut<B>>::Output;

    type InplaceOutput = Tensor<<A as NormalOut<B>>::Output>;

    fn matmul(&self, rhs: Tensor<B>) -> anyhow::Result<Self::Output> {
        Ok(matmul_with_out(self.inner.as_ref(), rhs.inner.as_ref(), None::<Self::Output>)?.into())
    }
    fn matmul_<U>(&self, rhs: Tensor<B>, out: U) -> anyhow::Result<Self::Output>
        where U: Borrow<Self::InplaceOutput> + BorrowMut<Self::InplaceOutput>
    {
        let out = out.borrow().inner.as_ref().clone();
        Ok(matmul_with_out(self.inner.as_ref(), rhs.inner.as_ref(), Some(out))?.into())
    }
}

impl<A, B> Matmul<&Tensor<B>>
    for Tensor<A>
    where
        A: CommonBounds + NormalOut<B> + IntoScalar<<A as NormalOut<B>>::Output>,
        B: CommonBounds + IntoScalar<<A as NormalOut<B>>::Output>,
        <A as NormalOut<B>>::Output: CommonBounds
{
    type Output = Tensor<<A as NormalOut<B>>::Output>;

    type OutputMeta = <A as NormalOut<B>>::Output;

    type InplaceOutput = Tensor<<A as NormalOut<B>>::Output>;

    fn matmul(&self, rhs: &Tensor<B>) -> anyhow::Result<Self::Output> {
        Ok(matmul_with_out(self.inner.as_ref(), rhs.inner.as_ref(), None::<Self::Output>)?.into())
    }

    fn matmul_<U>(&self, rhs: &Tensor<B>, out: U) -> anyhow::Result<Self::Output>
        where U: Borrow<Self::InplaceOutput> + BorrowMut<Self::InplaceOutput>
    {
        let out = out.borrow().inner.as_ref().clone();
        Ok(matmul_with_out(self.inner.as_ref(), rhs.inner.as_ref(), Some(out))?.into())
    }
}
