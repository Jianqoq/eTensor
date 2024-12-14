#[macro_use]
extern crate tensor_tests;
use tensor_dyn::*;
pub fn main() {
    fn case1(a: f32, b: f32) -> anyhow::Result<f32> {
        let Tensor { data: b } = a;
        Ok(a)
    }
    fn case1(a: f32, b: f32) -> anyhow::Result<f32> {
        let Tensor { data: b, shape: _, strides: _, offset: _ } = a;
        Ok(a)
    }
    fn case1(a: f32, b: f32) -> anyhow::Result<f32> {
        let Tensor { data: (b, b), shape: (_, _) } = a;
        Ok(a)
    }
    fn case1(a: f32, b: f32) -> anyhow::Result<f32> {
        let Tensor {
            data: (b, b),
            shape: (Tensor { data: (b, b), shape: (_, _) }, _),
        } = a;
        Ok(a)
    }
    fn case1(a: f32, b: f32) -> anyhow::Result<f32> {
        let (ok, lk) = b;
        Ok(a)
    }
    fn case1(a: f32, b: f32) -> anyhow::Result<f32> {
        let (ok) = a;
        Ok(a)
    }
    fn case1(a: f32, b: f32) -> anyhow::Result<f32> {
        let [ok, ok2] = a;
        Ok(a)
    }
    fn case1(a: f32, b: f32) -> anyhow::Result<f32> {
        let [ok, ok2, ..] = a;
        Ok(a)
    }
}
