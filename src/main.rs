use std::{env, fs};

pub mod cel;
pub mod poc;
pub mod runner;
pub mod template;

use crate::cel::{CelEnv, ResponseValue, UrlValue};
use crate::runner::Runner;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();

    // 检查是否提供了测试模式参数
    if args.len() > 1 && args[1] == "--test-cel" {
        test_cel_functions();
        return Ok(());
    }

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

    println!("加载成功, POC 名称: {}", poc.name);

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

/// 测试 CEL 函数实现
fn test_cel_functions() {
    println!("开始测试 CEL 函数实现...");

    // 创建测试环境
    let mut env = CelEnv::new();

    // 创建测试响应
    let mut response = ResponseValue::default();
    response.status = 200;
    response.body = b"Hello, Example Domain! This is a test response.".to_vec();
    response.body_string = String::from_utf8_lossy(&response.body).to_string();
    response.content_type = "text/html; charset=UTF-8".to_string();

    // 设置测试 URL
    let mut url = UrlValue::default();
    url.full = "https://example.com:8080/path/to/resource?param=value#fragment".to_string();
    url.scheme = "https".to_string();
    url.domain = "example.com".to_string();
    url.host = "example.com:8080".to_string();
    url.port = "8080".to_string();
    url.path = "/path/to/resource".to_string();
    url.query = "param=value".to_string();
    url.fragment = "fragment".to_string();
    response.url = url;

    // 添加测试响应头
    response.headers.insert(
        "Content-Type".to_string(),
        "text/html; charset=UTF-8".to_string(),
    );
    response
        .headers
        .insert("Server".to_string(), "Example Server".to_string());

    // 更新环境
    env.update_response(response);

    // 测试基本表达式
    test_expression(&env, "response_status == 200", true);
    test_expression(
        &env,
        "response_body_string.contains('Example Domain')",
        true,
    );
    test_expression(&env, "response_content_type.contains('text/html')", true);

    // 测试函数调用
    test_expression(&env, "contains(response_body_string, 'Example')", true);
    test_expression(&env, "startsWith(response_body_string, 'Hello')", true);
    test_expression(&env, "endsWith(response_body_string, 'not found')", false);

    // 测试转换后的表达式
    test_expression(&env, "response.body_string.contains('Example')", true);
    test_expression(&env, "response.status == 200", true);

    println!("CEL 函数测试完成！");
}

/// 测试表达式并验证结果
fn test_expression(env: &CelEnv, expression: &str, expected: bool) {
    match env.evaluate(expression) {
        Ok(value) => {
            if let cel_interpreter::Value::Bool(result) = value {
                if result == expected {
                    println!("✅ 表达式测试通过: {} => {}", expression, result);
                } else {
                    println!(
                        "❌ 表达式测试失败: {} => {}, 期望 {}",
                        expression, result, expected
                    );
                }
            } else {
                println!("❌ 表达式结果不是布尔值: {}", expression);
            }
        }
        Err(e) => {
            println!("❌ 表达式评估失败: {} - 错误: {}", expression, e);
        }
    }
}
