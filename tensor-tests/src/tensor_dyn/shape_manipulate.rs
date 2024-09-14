#![allow(unused_imports)]
use rayon::iter::{ IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator };
use tch::Tensor;
use tensor_dyn::{ tensor_base::_Tensor, TensorCreator };
use tensor_dyn::TensorInfo;
use tensor_dyn::ShapeManipulate;

#[allow(unused)]
fn assert_eq(b: &_Tensor<f64>, a: &Tensor) {
    let a_raw = if b.strides().contains(&0) {
        let size = b
            .shape()
            .iter()
            .zip(b.strides().iter())
            .filter(|(sp, s)| { **s != 0 })
            .fold(1, |acc, (sp, _)| acc * sp);
        unsafe { std::slice::from_raw_parts(a.data_ptr() as *const f64, size as usize) }
    } else {
        unsafe { std::slice::from_raw_parts(a.data_ptr() as *const f64, b.size()) }
    };
    let b_raw = b.as_raw();
    let tolerance = 2.5e-16;

    a_raw
        .iter()
        .zip(b_raw.iter())
        .for_each(|(a, b)| {
            let abs_diff = (a - b).abs();
            let relative_diff = abs_diff / b.abs().max(f64::EPSILON);

            if abs_diff > tolerance && relative_diff > tolerance {
                panic!("{} != {} (abs_diff: {}, relative_diff: {})", a, b, abs_diff, relative_diff);
            }
        });
}
#[test]
fn test_transpose() -> anyhow::Result<()> {
    let tch_a = tch::Tensor::randn(&[10, 10], (tch::Kind::Double, tch::Device::Cpu));
    let a = _Tensor::<f64>::empty(&[10, 10])?;
    a.as_raw_mut().copy_from_slice(unsafe {
        std::slice::from_raw_parts(tch_a.data_ptr() as *const f64, a.size())
    });
    let b = a.transpose(0, 1)?;
    let tch_b = tch_a.transpose(0, 1);
    assert_eq(&b, &tch_b);
    assert_eq!(&tch_b.size(), b.shape().inner());
    Ok(())
}

#[test]
fn test_unsqueeze() -> anyhow::Result<()> {
    let tch_a = tch::Tensor::randn(&[10], (tch::Kind::Double, tch::Device::Cpu));
    let a = _Tensor::<f64>::empty(&[10])?;
    a.as_raw_mut().copy_from_slice(unsafe {
        std::slice::from_raw_parts(tch_a.data_ptr() as *const f64, a.size())
    });
    let b = a.unsqueeze(0)?;
    let tch_b = tch_a.unsqueeze(0);
    let b = b.unsqueeze(1)?;
    let tch_b = tch_b.unsqueeze(1);
    assert_eq(&b, &tch_b);
    assert_eq!(&tch_b.size(), b.shape().inner());
    Ok(())
}

#[test]
fn test_squeeze() -> anyhow::Result<()> {
    let tch_a = tch::Tensor::randn(&[1, 10, 1], (tch::Kind::Double, tch::Device::Cpu));
    let a = _Tensor::<f64>::empty(&[1, 10, 1])?;
    a.as_raw_mut().copy_from_slice(unsafe {
        std::slice::from_raw_parts(tch_a.data_ptr() as *const f64, a.size())
    });
    let b = a.squeeze(0)?;
    let tch_b = tch_a.squeeze_dim(0);
    let b = b.squeeze(1)?;
    let tch_b = tch_b.squeeze_dim(1);
    assert_eq(&b, &tch_b);
    assert_eq!(&tch_b.size(), b.shape().inner());
    Ok(())
}

#[test]
fn test_expand() -> anyhow::Result<()> {
    let tch_a = tch::Tensor::randn(&[1, 10, 1], (tch::Kind::Double, tch::Device::Cpu));
    let a = _Tensor::<f64>::empty(&[1, 10, 1])?;
    a.as_raw_mut().copy_from_slice(unsafe {
        std::slice::from_raw_parts(tch_a.data_ptr() as *const f64, a.size())
    });
    let b = a.expand(&[10, 10, 10])?;
    let tch_b = tch_a.expand(&[10, 10, 10], true);
    assert_eq(&b, &tch_b);
    assert_eq!(&tch_b.size(), b.shape().inner());
    Ok(())
}

#[test]
fn test_flatten() -> anyhow::Result<()> {
    let tch_a = tch::Tensor::randn(&[10, 10, 10], (tch::Kind::Double, tch::Device::Cpu));
    let a = _Tensor::<f64>::empty(&[10, 10, 10])?;
    a.as_raw_mut().copy_from_slice(unsafe {
        std::slice::from_raw_parts(tch_a.data_ptr() as *const f64, a.size())
    });
    let b = a.flatten(1, 2)?;
    let tch_b = tch_a.flatten(1, 2);
    assert_eq(&b, &tch_b);
    assert_eq!(&tch_b.size(), b.shape().inner());

    let b = a.flatten(0, 2)?;
    let tch_b = tch_a.flatten(0, 2);
    assert_eq(&b, &tch_b);
    assert_eq!(&tch_b.size(), b.shape().inner());

    let b = a.flatten(2, 2)?;
    let tch_b = tch_a.flatten(2, 2);
    assert_eq(&b, &tch_b);
    assert_eq!(&tch_b.size(), b.shape().inner());

    let b = a.flatten(0, 1)?;
    let tch_b = tch_a.flatten(0, 1);
    assert_eq(&b, &tch_b);
    assert_eq!(&tch_b.size(), b.shape().inner());

    Ok(())
}