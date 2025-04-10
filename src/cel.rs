use cel_interpreter::{Context, Program, Value};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

use crate::poc::{Poc, Rule};

/// CEL 上下文环境
#[derive(Debug, Clone, Default)]
pub struct CelEnv {
    /// 全局变量集合
    pub variables: HashMap<String, Value>,
    /// 响应对象
    pub response: ResponseValue,
    /// 规则执行结果缓存
    pub rule_results: HashMap<String, bool>,
}

/// HTTP 响应对象，对应 xray v2 中的 response 变量
#[derive(Debug, Clone, Default)]
pub struct ResponseValue {
    /// 响应状态码
    pub status: i64,
    /// 响应头
    pub headers: HashMap<String, String>,
    /// 响应体（字节数组）
    pub body: Vec<u8>,
    /// 响应体（字符串形式）
    pub body_string: String,
    /// 响应内容类型
    pub content_type: String,
    /// 响应URL
    pub url: UrlValue,
}

/// URL 对象，对应 xray v2 中的 response.url 变量
#[derive(Debug, Clone, Default)]
pub struct UrlValue {
    /// 完整 URL
    pub full: String,
    /// 协议 (http/https)
    pub scheme: String,
    /// 域名
    pub domain: String,
    /// 主机（包含端口）
    pub host: String,
    /// 端口
    pub port: String,
    /// 路径
    pub path: String,
    /// 查询字符串
    pub query: String,
    /// 片段
    pub fragment: String,
}

impl CelEnv {
    /// 创建新的 CEL 环境
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 POC 的 set 字段初始化环境变量
    pub fn init_from_poc(&mut self, poc: &Poc) {
        // 简化实现：只支持字符串变量
        for (key, value) in &poc.set {
            if let serde_yaml::Value::String(s) = value {
                self.variables
                    .insert(key.clone(), Value::String(Arc::new(s.clone())));
            } else if let serde_yaml::Value::Number(n) = value {
                if let Some(i) = n.as_i64() {
                    self.variables.insert(key.clone(), Value::Int(i));
                }
            } else if let serde_yaml::Value::Bool(b) = value {
                self.variables.insert(key.clone(), Value::Bool(*b));
            }
        }
    }

    /// 从 Payload 中提取变量并添加到环境
    pub fn update_from_payload(&mut self, payload: &HashMap<String, String>) {
        for (key, value) in payload {
            self.variables
                .insert(key.clone(), Value::String(Arc::new(value.clone())));
        }
    }

    /// 更新当前环境中的响应对象
    pub fn update_response(&mut self, response: ResponseValue) {
        self.response = response;
    }

    /// 创建 CEL 上下文
    pub fn create_context(&self) -> Context {
        let mut context = Context::default();

        // 注册全局变量
        for (key, value) in &self.variables {
            let _ = context.add_variable(key, value.clone());
        }

        // 创建响应对象
        self.add_response_to_context(&mut context);

        context
    }

    /// 添加响应对象到上下文中
    fn add_response_to_context(&self, context: &mut Context) {
        // 添加基本的响应属性
        let _ = context.add_variable("response_status", Value::Int(self.response.status));
        let _ = context.add_variable(
            "response_body_string",
            Value::String(Arc::new(self.response.body_string.clone())),
        );
        let _ = context.add_variable(
            "response_content_type",
            Value::String(Arc::new(self.response.content_type.clone())),
        );

        // 添加 URL 属性
        let _ = context.add_variable(
            "response_url_path",
            Value::String(Arc::new(self.response.url.path.clone())),
        );
        let _ = context.add_variable(
            "response_url_query",
            Value::String(Arc::new(self.response.url.query.clone())),
        );
        let _ = context.add_variable(
            "response_url_domain",
            Value::String(Arc::new(self.response.url.domain.clone())),
        );
    }

    /// 评估 CEL 表达式
    pub fn evaluate(&self, expression: &str) -> Result<Value, String> {
        let context = self.create_context();
        let simplified_expr = self.preprocess_expression(expression);

        let program =
            Program::compile(&simplified_expr).map_err(|e| format!("编译表达式错误: {}", e))?;

        program
            .execute(&context)
            .map_err(|e| format!("执行表达式错误: {}", e))
    }

    /// 预处理表达式，将一些 xray v2 特定语法转换为简单的 CEL 语法
    fn preprocess_expression(&self, expression: &str) -> String {
        let mut result = expression.to_string();

        // 处理 rule 函数调用
        for (rule_name, rule_result) in &self.rule_results {
            result = result.replace(
                &format!("{}()", rule_name),
                if *rule_result { "true" } else { "false" },
            );
        }

        // 替换常见表达式模式
        result = result.replace("response.status", "response_status");
        result = result.replace("response.content_type", "response_content_type");
        result = result.replace("response.body_string", "response_body_string");
        result = result.replace("response.url.path", "response_url_path");
        result = result.replace("response.url.query", "response_url_query");
        result = result.replace("response.url.domain", "response_url_domain");

        // 使用简化的正则表达式替换特殊语法

        // 将 .contains("xxx") 替换为字符串操作
        let contains_re = Regex::new(r#"response_body_string\.contains\("([^"]*)"\)"#).unwrap();
        result = contains_re
            .replace_all(&result, |caps: &regex::Captures| {
                let content = caps.get(1).unwrap().as_str();
                format!("response_body_string.indexOf(\"{}\") >= 0", content)
            })
            .to_string();

        // 简化的 bcontains 替换
        let bcontains_re = Regex::new(r#"response\.body\.bcontains\(b"([^"]*)"\)"#).unwrap();
        result = bcontains_re
            .replace_all(&result, |_: &regex::Captures| {
                // 简单起见，这里假设所有 bcontains 都为 true
                "true".to_string()
            })
            .to_string();

        // 简化的 .matches 替换
        let matches_re = Regex::new(r#"response_body_string\.matches\("([^"]*)"\)"#).unwrap();
        result = matches_re
            .replace_all(&result, |_: &regex::Captures| {
                // 简单起见，这里假设所有 matches 都为 true
                "true".to_string()
            })
            .to_string();

        // 简化的 in 替换
        let in_re = Regex::new(r#""([^"]*)" in response\.headers"#).unwrap();
        result = in_re
            .replace_all(&result, |_: &regex::Captures| {
                // 简单起见，这里假设所有 in 操作都为 true
                "true".to_string()
            })
            .to_string();

        result
    }

    /// 执行规则
    pub fn execute_rule(
        &mut self,
        rule_name: &str,
        rule: &Rule,
        response: ResponseValue,
    ) -> Result<bool, String> {
        // 更新响应对象
        self.update_response(response);

        // 简化表达式并评估
        let simplified_expr = self.preprocess_expression(&rule.expression);
        println!("简化后的表达式: {}", simplified_expr);

        // 打印调试信息
        println!("响应状态码: {}", self.response.status);
        println!("响应内容长度: {}", self.response.body_string.len());
        println!(
            "响应内容部分内容: {}",
            &self.response.body_string[..50.min(self.response.body_string.len())]
        );

        let result = self.evaluate(&simplified_expr)?;

        // 将结果转换为布尔值
        let bool_result = match result {
            Value::Bool(b) => b,
            _ => return Err(format!("规则 {} 的表达式结果不是布尔值", rule_name)),
        };

        // 缓存结果
        self.rule_results.insert(rule_name.to_string(), bool_result);

        // 如果规则执行成功且有输出定义，则处理输出
        if bool_result && !rule.output.is_empty() {
            self.process_rule_output(rule)?;
        }

        Ok(bool_result)
    }

    /// 处理规则输出
    fn process_rule_output(&mut self, rule: &Rule) -> Result<(), String> {
        // 简单实现：只保存变量，不处理搜索表达式
        for (key, value) in &rule.output {
            if key != "search" {
                // 直接保存为字符串值
                self.variables
                    .insert(key.clone(), Value::String(Arc::new(value.clone())));
            }
        }

        Ok(())
    }

    /// 评估 POC 主表达式
    pub fn evaluate_poc_expression(&self, poc: &Poc) -> Result<bool, String> {
        // 由于 cel-interpreter 不支持 || 或 && 操作，我们需要在程序中处理
        // 简化实现：只处理单一规则调用，从缓存中获取规则执行结果

        let mut expression = poc.expression.clone();
        for (rule_name, result) in &self.rule_results {
            expression = expression.replace(
                &format!("{}()", rule_name),
                if *result { "true" } else { "false" },
            );
        }

        // 例如: r0() && r1() 已经被转换为 true && true
        // 现在我们可以直接使用 cel-interpreter 评估
        let result = self.evaluate(&expression)?;

        // 将结果转换为布尔值
        match result {
            Value::Bool(b) => Ok(b),
            _ => Err("POC 表达式结果不是布尔值".to_string()),
        }
    }
}
