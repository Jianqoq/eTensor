use into_scalar::IntoScalar;
use rust_xlsxwriter::{ Format, Workbook };
use std::io::Write;
use tensor_dyn::tensor_base::_Tensor;
use tensor_dyn::*;


// #[compile]
// fn compute2<T: CommonBounds>(a: _Tensor<T>, b: _Tensor<T>, k: f32) -> anyhow::Result<_Tensor<T>>
//     where
//         T: FloatOutUnary<Output = T, Base = T>,
//         T::Vec: FloatOutUnary<Output = T::Vec, Base = T>,
//         T: NormalOutUnary<Base = T>,
//         T::Vec: NormalOutUnary<Base = T>,
//         f64: IntoScalar<T>,
//         Option<T>: From<f64>
// {
//     if a > 0.0 {
//         a = 10
//     } else if a > 0.0 {
//         a = 20
//     } else {
//         a = 30
//     }
//     Ok(c)
// }

fn main() -> anyhow::Result<()> {
    // conv2d()?;
    // let a = _Tensor::<f32>::arange(0, 10000)?;
    // let b = _Tensor::<f32>::arange(0, 10000)?;
    // let now = std::time::Instant::now();
    // let c = compute(a, b)?;
    Ok(())
}

fn conv2d() -> Result<(), anyhow::Error> {
    let oc_sets = [128, 256, 512, 1024, 2048, 4096, 8192];
    let ic_sets = [8192];
    let kh_sets = [4];
    let kw_sets = [4];
    let h_sets = [256];
    let w_sets = [256];

    // set_num_threads(10);
    let mut workbook = Workbook::new();
    let decimal_format = Format::new().set_num_format("0.0000000000");
    let format = Format::new();
    let worksheet = workbook.add_worksheet();

    let mut row = 0;
    for ic in ic_sets {
        for kh in kh_sets {
            for kw in kw_sets {
                for h in h_sets {
                    for w in w_sets {
                        let a = _Tensor::<f32>
                            ::arange(0, 1 * ic * h * w)?
                            .reshape([1, ic, h, w])?
                            .permute([0, 2, 3, 1])?
                            .contiguous()?;
                        // let device = Device::Cpu;
                        // let a = Tensor::randn(1.0, 1.0, &[1, ic, h, w], &device)?;
                        // let kernel = Tensor::randn(1.0, 1.0, &[oc, ic, kh, kw], &device)?;
                        let now = std::time::Instant::now();
                        for _ in 0..10 {
                            let _ = a.maxpool2d(
                                &[kh, kw].into(),
                                [1, 1],
                                [
                                    (0, 0),
                                    (0, 0),
                                ],
                                [2, 2]
                            )?;
                        }
                        println!("{:?}", now.elapsed() / 10);
                        worksheet.write_number(
                            row,
                            0,
                            (now.elapsed().as_millis() as f64) / 10.0,
                            &decimal_format
                        )?;
                        worksheet.write_string(
                            row,
                            1,
                            &format!("({}, {}, {}, {}, {})", ic, kh, kw, h, w),
                            &format
                        )?;
                        print!(
                            "\rprogress: {}%",
                            ((row + 1) * 100) /
                                (
                                    (ic_sets.len() *
                                        oc_sets.len() *
                                        kh_sets.len() *
                                        kw_sets.len() *
                                        h_sets.len() *
                                        w_sets.len()) as u32
                                )
                        );
                        std::io::stdout().flush().expect("Failed to flush stdout");
                        row += 1;
                    }
                }
            }
        }
    }

    workbook.save("conv2d_result.xlsx")?;
    Ok(())
}
