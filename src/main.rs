use std::{env, fs};

pub mod cel;
pub mod poc;
pub mod runner;
pub mod template;

use crate::runner::Runner;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    // 检查是否提供了 POC 文件路径
    let poc_path = if args.len() > 1 {
        &args[1]
    } else {
        "poc.yaml" // 默认使用当前目录下的 poc.yaml
    };

    println!("正在加载 POC 文件: {}", poc_path);

    // 打开并解析 POC 文件
    let file = fs::File::open(poc_path)?;
    let poc: poc::Poc = serde_yaml::from_reader(file)?;

    println!("加载成功，POC 名称: {}", poc.name);

    // 创建并运行 POC
    let runner = Runner::new(poc);
    match runner.run() {
        Ok(result) => {
            if result.success {
                println!("POC 执行成功: {}", result.message);

                // 输出变量
                if !result.output.is_empty() {
                    println!("输出变量:");
                    for (key, value) in &result.output {
                        println!("  {}: {}", key, value);
                    }
                }
            } else {
                println!("POC 执行失败: {}", result.message);
            }
        }
        Err(e) => {
            println!("POC 执行错误: {}", e);
        }
    }

    Ok(())
}
