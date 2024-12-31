use tensor_dyn::*;

fn main() -> anyhow::Result<()> {
    Ok(())
}

// fn visit_dirs(dir: &str) -> std::io::Result<()> {
//     let mut saver = TensorSaver::new("weights/data.ftz".into());
//     // 读取当前目录的条目
//     for entry_result in std::fs::read_dir(dir)? {
//         let entry = entry_result?;
//         let file_name = entry.file_name();
//         let path = entry.path();

//         // 如果是目录，递归调用
//         if path.is_dir() {
//             visit_dirs(path.to_str().unwrap())?;
//         } else {
//             let file = std::fs::File::open(&path)?;
//             let reader = std::io::BufReader::new(file);
//             let mut lines = reader.lines();
//             let shape = lines.next().unwrap().unwrap();
//             let after_shape = shape.strip_prefix("shape:").unwrap().trim();
//             let mut shape = Vec::new();
//             if !after_shape.is_empty() {
//                 let tokens = after_shape.split_whitespace();
//                 for token in tokens {
//                     // 尝试解析为整数
//                     if let Ok(val) = token.parse::<i64>() {
//                         shape.push(val);
//                     }
//                 }
//             }
//             println!("{}: {:?}", file_name.to_str().unwrap(), shape);
//             let mut data = Vec::new();
//             while let Some(line) = lines.next() {
//                 let line = line?;
//                 let tokens = line.split_whitespace();
//                 for token in tokens {
//                     if let Ok(val) = token.parse::<f32>() {
//                         data.push(val);
//                     }
//                 }
//             }
//             if shape.len() > 0 {
//                 let tensor = Tensor::<f32>::new(data.clone()).reshape(shape).unwrap();
//                 saver = saver.push(
//                     file_name.to_str().unwrap(),
//                     tensor,
//                     CompressionAlgo::Gzip,
//                     Endian::Native,
//                     9,
//                 );
//             }
//         }
//     }
//     saver.save()?;
//     Ok(())
// }

// fn load_dirs(dir: &str) -> std::io::Result<()> {
//     // 读取当前目录的条目
//     for entry_result in std::fs::read_dir(dir)? {
//         let entry = entry_result?;
//         let file_name = entry.file_name();
//         let path = entry.path();

//         // 如果是目录，递归调用
//         if path.is_dir() {
//             visit_dirs(path.to_str().unwrap())?;
//         } else {
//             let file = std::fs::File::open(&path)?;
//             let reader = std::io::BufReader::new(file);
//             let mut lines = reader.lines();
//             let shape = lines.next().unwrap().unwrap();
//             let after_shape = shape.strip_prefix("shape:").unwrap().trim();
//             let mut shape = Vec::new();
//             if !after_shape.is_empty() {
//                 let tokens = after_shape.split_whitespace();
//                 for token in tokens {
//                     // 尝试解析为整数
//                     if let Ok(val) = token.parse::<i64>() {
//                         shape.push(val);
//                     }
//                 }
//             }
//             println!("{}: {:?}", file_name.to_str().unwrap(), shape);
//             let mut data = Vec::new();
//             while let Some(line) = lines.next() {
//                 let line = line?;
//                 let tokens = line.split_whitespace();
//                 for token in tokens {
//                     if let Ok(val) = token.parse::<f32>() {
//                         data.push(val);
//                     }
//                 }
//             }
//             if shape.len() > 0 {
//                 let loader = TensorLoader::new(r"C:\Users\123\eTensor-1\weights\data.ftz".into());
//                 let res = loader
//                     .push(file_name.to_str().unwrap(), &[])
//                     .load::<f32, Tensor<f32>, 4>()?;
//                 println!("{}", res[0]);
//                 std::thread::sleep(std::time::Duration::from_millis(1000));
//             }
//         }
//     }
//     Ok(())
// }
