use std::fs;

pub mod poc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open("poc.yaml")?;

    // 将 YAML 字符串反序列化为 Poc 结构体
    let poc: poc::Poc = serde_yaml::from_reader(file)?;
    // 打印反序列化后的结构体
    println!("完整 POC 示例: {:#?}", poc);

    Ok(())
}
