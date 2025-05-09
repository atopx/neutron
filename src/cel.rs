use cel_interpreter::objects::Map;
use cel_interpreter::{Context, Program, Value};
use regex::Regex;
use std::collections::HashMap;
use std::mem;
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
    /// 是否重定向
    pub redirect: bool,
    /// 响应时间(ms)
    pub latency: i64,
    /// 全部响应
    pub raw: Vec<u8>,
    /// 全部响应（字符串形式）
    pub raw_string: String,
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

        // 添加响应对象属性
        self.add_response_to_context(&mut context);

        context
    }

    /// 添加响应对象到上下文中
    fn add_response_to_context(&self, context: &mut Context) {
        // 添加基本的响应属性（扁平化，而不是嵌套对象）
        let _ = context.add_variable("response_status", Value::Int(self.response.status));
        let _ = context.add_variable(
            "response_body_string",
            Value::String(Arc::new(self.response.body_string.clone())),
        );
        let _ = context.add_variable(
            "response_content_type",
            Value::String(Arc::new(self.response.content_type.clone())),
        );
        let _ = context.add_variable("response_redirect", Value::Bool(self.response.redirect));
        let _ = context.add_variable("response_latency", Value::Int(self.response.latency));

        // 添加响应体二进制形式
        let _ = context.add_variable(
            "response_body",
            Value::Bytes(Arc::new(self.response.body.clone())),
        );

        // 添加 URL 属性
        let _ = context.add_variable(
            "response_url_full",
            Value::String(Arc::new(self.response.url.full.clone())),
        );
        let _ = context.add_variable(
            "response_url_scheme",
            Value::String(Arc::new(self.response.url.scheme.clone())),
        );
        let _ = context.add_variable(
            "response_url_domain",
            Value::String(Arc::new(self.response.url.domain.clone())),
        );
        let _ = context.add_variable(
            "response_url_host",
            Value::String(Arc::new(self.response.url.host.clone())),
        );
        let _ = context.add_variable(
            "response_url_port",
            Value::String(Arc::new(self.response.url.port.clone())),
        );
        let _ = context.add_variable(
            "response_url_path",
            Value::String(Arc::new(self.response.url.path.clone())),
        );
        let _ = context.add_variable(
            "response_url_query",
            Value::String(Arc::new(self.response.url.query.clone())),
        );
        let _ = context.add_variable(
            "response_url_fragment",
            Value::String(Arc::new(self.response.url.fragment.clone())),
        );

        // 添加 headers
        for (key, value) in &self.response.headers {
            let var_name = format!("response_headers_{}", key.to_lowercase().replace("-", "_"));
            let _ = context.add_variable(&var_name, Value::String(Arc::new(value.clone())));
        }
    }

    /// 注册并调用额外的函数
    ///
    /// 注意: 这些函数应该在 preprocess_expression 中通过表达式转换来支持，
    /// 而不是通过函数注册，因为 cel-interpreter 的函数注册机制较为复杂
    fn call_custom_function(&self, function: &str, args: &[Value]) -> Result<Value, String> {
        match function {
            "contains" => {
                if args.len() != 2 {
                    return Err("contains 函数需要两个参数".to_string());
                }
                if let (Value::String(haystack), Value::String(needle)) = (&args[0], &args[1]) {
                    Ok(Value::Bool(haystack.contains(needle.as_ref())))
                } else {
                    Err("参数类型错误".to_string())
                }
            }
            "starts_with" => {
                if args.len() != 2 {
                    return Err("starts_with 函数需要两个参数".to_string());
                }
                if let (Value::String(haystack), Value::String(needle)) = (&args[0], &args[1]) {
                    Ok(Value::Bool(haystack.starts_with(needle.as_ref())))
                } else {
                    Err("参数类型错误".to_string())
                }
            }
            "ends_with" => {
                if args.len() != 2 {
                    return Err("ends_with 函数需要两个参数".to_string());
                }
                if let (Value::String(haystack), Value::String(needle)) = (&args[0], &args[1]) {
                    Ok(Value::Bool(haystack.ends_with(needle.as_ref())))
                } else {
                    Err("参数类型错误".to_string())
                }
            }
            "matches" => {
                if args.len() != 2 {
                    return Err("matches 函数需要两个参数".to_string());
                }
                if let (Value::String(pattern), Value::String(text)) = (&args[0], &args[1]) {
                    match Regex::new(pattern.as_ref()) {
                        Ok(re) => Ok(Value::Bool(re.is_match(text.as_ref()))),
                        Err(e) => Err(format!("正则表达式错误: {}", e)),
                    }
                } else {
                    Err("参数类型错误".to_string())
                }
            }
            "bcontains" => {
                if args.len() != 2 {
                    return Err("bcontains 函数需要两个参数".to_string());
                }

                let (haystack, needle) = match (&args[0], &args[1]) {
                    (Value::Bytes(h), Value::Bytes(n)) => (h.as_ref(), n.as_ref()),
                    (Value::String(h), Value::String(n)) => (h.as_bytes(), n.as_bytes()),
                    _ => return Err("参数类型错误".to_string()),
                };

                Ok(Value::Bool(
                    memchr::memmem::find(haystack, needle).is_some(),
                ))
            }
            _ => Err(format!("不支持的函数: {}", function)),
        }
    }

    /// 评估 CEL 表达式
    pub fn evaluate(&self, expression: &str) -> Result<Value, String> {
        let context = self.create_context();
        let simplified_expr = self.preprocess_expression(expression);

        println!("原始表达式: {}", expression);
        println!("预处理后表达式: {}", simplified_expr);

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

        // 替换 Xray 特有的语法到 CEL 表达式

        // 响应对象属性访问转换
        result = result.replace("response.status", "response_status");
        result = result.replace("response.body_string", "response_body_string");
        result = result.replace("response.content_type", "response_content_type");
        result = result.replace("response.redirect", "response_redirect");
        result = result.replace("response.latency", "response_latency");
        result = result.replace("response.body", "response_body");

        // URL 属性访问转换
        result = result.replace("response.url.full", "response_url_full");
        result = result.replace("response.url.scheme", "response_url_scheme");
        result = result.replace("response.url.domain", "response_url_domain");
        result = result.replace("response.url.host", "response_url_host");
        result = result.replace("response.url.port", "response_url_port");
        result = result.replace("response.url.path", "response_url_path");
        result = result.replace("response.url.query", "response_url_query");
        result = result.replace("response.url.fragment", "response_url_fragment");

        // 1. 字符串方法调用转换为函数调用
        // x.contains("xxx") => contains(x, "xxx")
        let contains_re = Regex::new(r#"([^\.]+)\.contains\((.+?)\)"#).unwrap();
        result = contains_re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("contains({}, {})", &caps[1], &caps[2])
            })
            .to_string();

        // startsWith 转换
        let starts_with_re = Regex::new(r#"([^\.]+)\.startsWith\((.+?)\)"#).unwrap();
        result = starts_with_re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("starts_with({}, {})", &caps[1], &caps[2])
            })
            .to_string();

        // endsWith 转换
        let ends_with_re = Regex::new(r#"([^\.]+)\.endsWith\((.+?)\)"#).unwrap();
        result = ends_with_re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("ends_with({}, {})", &caps[1], &caps[2])
            })
            .to_string();

        // 2. 二进制方法调用转换
        // x.bcontains(b"xxx") => contains(x, "xxx")
        let bcontains_re = Regex::new(r#"([^\.]+)\.bcontains\(b"([^"]*)"\)"#).unwrap();
        result = bcontains_re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("contains({}, \"{}\")", &caps[1], &caps[2])
            })
            .to_string();

        // 3. 正则匹配转换
        // "xxx".matches(y) => contains(y, "xxx")
        let matches_re = Regex::new(r#""([^"]*)"\.matches\((.+?)\)"#).unwrap();
        result = matches_re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("contains({}, \"{}\")", &caps[2], &caps[1])
            })
            .to_string();

        // 4. 响应头处理
        // response.headers["Content-Type"] => response_headers_content_type
        let headers_re = Regex::new(r#"response\.headers\["([^"]*)"\]"#).unwrap();
        result = headers_re
            .replace_all(&result, |caps: &regex::Captures| {
                let header_name = caps[1].to_lowercase().replace("-", "_");
                format!("response_headers_{}", header_name)
            })
            .to_string();

        // 5. in 操作符转换
        // "xxx" in response.headers => response_headers_xxx != ""
        let in_re = Regex::new(r#""([^"]*)" in response\.headers"#).unwrap();
        result = in_re
            .replace_all(&result, |caps: &regex::Captures| {
                let header_name = caps[1].to_lowercase().replace("-", "_");
                format!("response_headers_{} != \"\"", header_name)
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

        // 预处理表达式并评估
        let simplified_expr = self.preprocess_expression(&rule.expression);
        println!("规则 {} 简化后的表达式: {}", rule_name, simplified_expr);

        // 打印调试信息
        println!("响应状态码: {}", self.response.status);
        println!("响应内容长度: {}", self.response.body_string.len());
        if !self.response.body_string.is_empty() {
            println!(
                "响应内容部分内容: {}",
                &self.response.body_string[..50.min(self.response.body_string.len())]
            );
        }

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
        for (key, value) in &rule.output {
            if key == "search" {
                // 处理特殊的 search 字段，执行正则匹配
                let re_pattern = value;
                let target_text = &self.response.body_string;

                match Regex::new(re_pattern) {
                    Ok(re) => {
                        if let Some(caps) = re.captures(target_text) {
                            // 检查是否有命名捕获组
                            for name in re.capture_names().flatten() {
                                if let Some(m) = caps.name(name) {
                                    self.variables.insert(
                                        name.to_string(),
                                        Value::String(Arc::new(m.as_str().to_string())),
                                    );
                                }
                            }

                            // 处理数字捕获组
                            for i in 1..caps.len() {
                                if let Some(m) = caps.get(i) {
                                    self.variables.insert(
                                        format!("{}", i),
                                        Value::String(Arc::new(m.as_str().to_string())),
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => return Err(format!("正则表达式编译错误: {}", e)),
                }
            } else {
                // 处理普通字段，支持引用其他变量
                let rendered_value = self.render_output_value(value)?;
                self.variables
                    .insert(key.clone(), Value::String(Arc::new(rendered_value)));
            }
        }

        Ok(())
    }

    /// 渲染输出值，支持引用其他变量
    fn render_output_value(&self, template: &str) -> Result<String, String> {
        // 简单的模板渲染实现
        let re = Regex::new(r"\{\{([^}]+)\}\}").unwrap();

        let result = re.replace_all(template, |caps: &regex::Captures| {
            let var_name = caps.get(1).unwrap().as_str().trim();

            if let Some(value) = self.variables.get(var_name) {
                match value {
                    Value::String(s) => s.to_string(),
                    Value::Int(i) => i.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => format!("{:?}", value),
                }
            } else if var_name.starts_with("response.") {
                // 支持引用响应对象的属性
                let parts: Vec<&str> = var_name.split('.').collect();
                if parts.len() == 2 && parts[1] == "status" {
                    self.response.status.to_string()
                } else if parts.len() == 2 && parts[1] == "body_string" {
                    self.response.body_string.clone()
                } else if parts.len() == 3 && parts[1] == "headers" {
                    self.response
                        .headers
                        .get(parts[2])
                        .cloned()
                        .unwrap_or_default()
                } else if parts.len() == 3 && parts[1] == "url" && parts[2] == "path" {
                    self.response.url.path.clone()
                } else {
                    format!("{{{{{}}}}}", var_name)
                }
            } else {
                format!("{{{{{}}}}}", var_name)
            }
        });

        Ok(result.to_string())
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
