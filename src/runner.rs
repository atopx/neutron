use cel_interpreter::Value;
use std::collections::HashMap;

use crate::cel::{CelEnv, ResponseValue};
use crate::poc::{Poc, Rule};
use crate::template;

/// POC 执行结果
#[derive(Debug, Clone, Default)]
pub struct RunResult {
    /// 是否成功
    pub success: bool,
    /// 输出变量
    pub output: HashMap<String, String>,
    /// 信息
    pub message: String,
}

/// POC 运行器
#[derive(Debug)]
pub struct Runner {
    /// POC 定义
    poc: Poc,
}

impl Runner {
    /// 创建新的 POC 运行器
    pub fn new(poc: Poc) -> Self {
        Self { poc }
    }

    /// 执行 POC
    pub fn run(&self) -> Result<RunResult, String> {
        let mut env = CelEnv::new();
        env.init_from_poc(&self.poc);

        // 如果有 payload，则循环执行每个 payload
        if !self.poc.payloads.payloads.is_empty() {
            return self.run_with_payloads(&mut env);
        }

        // 否则，直接执行规则
        self.run_rules(&mut env)
    }

    /// 使用 payload 执行 POC
    fn run_with_payloads(&self, env: &mut CelEnv) -> Result<RunResult, String> {
        let mut result = RunResult::default();

        // 克隆数据以避免借用问题
        let payloads = self.poc.payloads.payloads.clone();
        let continue_flag = self.poc.payloads.continue_;

        for (payload_name, payload_vars) in payloads.iter() {
            // 备份环境变量
            let env_vars_backup = env.variables.clone();

            // 将 payload 变量添加到环境
            env.update_from_payload(payload_vars);

            // 执行规则
            let rule_result = self.run_rules(env)?;

            // 恢复环境变量
            env.variables = env_vars_backup;

            // 如果成功且不需要继续，则返回
            if rule_result.success && !continue_flag {
                let mut updated_result = rule_result;
                updated_result.message = format!("POC 执行成功（使用 payload: {}）", payload_name);
                return Ok(updated_result);
            }

            // 如果成功，更新结果
            if rule_result.success {
                result = rule_result;
                result.success = true;
            }
        }

        if result.success {
            result.message = "POC 执行成功（至少一个 payload 匹配）".to_string();
        } else {
            result.message = "POC 执行失败（所有 payload 均未匹配）".to_string();
        }

        Ok(result)
    }

    /// 执行规则
    fn run_rules(&self, env: &mut CelEnv) -> Result<RunResult, String> {
        let mut result = RunResult::default();
        let expression = self.poc.expression.clone();

        // 解析表达式中的规则调用
        let rule_calls = extract_rule_calls(&expression);

        // 按照表达式中的顺序执行规则
        for rule_name in rule_calls {
            if let Some(rule) = self.poc.rules.get(&rule_name).cloned() {
                // 创建一个规则副本，以便进行模板渲染
                let mut rule_copy = rule.clone();

                // 渲染请求中的模板变量
                template::render_request(&mut rule_copy.request, &env.variables);

                // 模拟执行 HTTP 请求并获取响应
                // 在实际应用中，这里应该执行真正的 HTTP 请求
                let response = self.mock_http_request(&rule_copy)?;

                // 执行规则
                let rule_result = env.execute_rule(&rule_name, &rule_copy, response)?;

                // 如果规则失败且表达式使用 && 连接，则可以提前结束
                if !rule_result
                    && expression.contains(&format!("{}()", rule_name))
                    && expression.contains("&&")
                {
                    result.success = false;
                    result.message = format!("规则 {} 执行失败", rule_name);
                    return Ok(result);
                }
            } else {
                return Err(format!("规则 {} 不存在", rule_name));
            }
        }

        // 评估整个表达式
        let expr_result = env.evaluate_poc_expression(&self.poc)?;

        // 设置结果
        result.success = expr_result;
        if expr_result {
            result.message = "POC 执行成功".to_string();

            // 收集输出变量
            for (name, value) in &env.variables {
                if let Value::String(s) = value {
                    // 将 Arc<String> 转换为 String
                    let string_value = s.to_string();
                    result.output.insert(name.clone(), string_value);
                }
            }

            // 如果有详情字段，则渲染模板变量
            let mut poc_copy = self.poc.clone();
            template::render_detail(&mut poc_copy.detail, &env.variables);

            // 如果有漏洞匹配信息，则添加到输出
            let match_info = &poc_copy.detail.vulnerability.r#match;
            if !match_info.is_empty() {
                result
                    .output
                    .insert("match_info".to_string(), match_info.to_string());
            }
        } else {
            result.message = "POC 执行失败".to_string();
        }

        Ok(result)
    }

    /// 模拟 HTTP 请求（实际应用中应替换为真正的 HTTP 请求）
    fn mock_http_request(&self, rule: &Rule) -> Result<ResponseValue, String> {
        // 这里只是一个模拟实现，实际应用中应该发送真正的 HTTP 请求
        let method = &rule.request.method;
        let path = &rule.request.path;

        println!("模拟请求: {} {}", method, path);

        // 简单的响应模拟，根据路径返回不同的内容
        let mut response = ResponseValue::default();
        response.status = 200;
        response
            .headers
            .insert("Content-Type".to_string(), "text/plain".to_string());

        // 针对 CVE-2021-43798 的测试响应
        if path.contains("/public/plugins/") && path.contains("passwd") {
            // 模拟一个包含 /etc/passwd 内容的响应
            response.body = br#"root:x:0:0:root:/root:/bin/bash
daemon:x:1:1:daemon:/usr/sbin:/usr/sbin/nologin
bin:x:2:2:bin:/bin:/usr/sbin/nologin
sys:x:3:3:sys:/dev:/usr/sbin/nologin
sync:x:4:65534:sync:/bin:/bin/sync
xxxxx"#
                .to_vec();
            response.content_type = "text/plain".to_string();
        }
        // 模拟不同路径的响应
        else if path.contains("actuator") {
            if path.contains("env") {
                response.body = br#"{"activeProfiles":[],"propertySources":[{"name":"server.ports","properties":{"local.server.port":{"value":8080}}},{"name":"systemProperties","properties":{"java.version":{"value":"11.0.12"}}}]}"#.to_vec();
            } else if path.contains("health") {
                response.body = br#"{"status":"UP"}"#.to_vec();
            } else if path.contains("info") {
                response.body = br#"{"app":{"name":"spring-boot-app","version":"1.0.0"}}"#.to_vec();
            } else {
                response.body = br#"{"_links":{"self":{"href":"http://localhost:8080/actuator","templated":false},"health":{"href":"http://localhost:8080/actuator/health","templated":false}}}"#.to_vec();
            }
            response.content_type = "application/json".to_string();
        } else if path.contains("vulnerable.php") {
            if path.contains("1=1") {
                response.body = br#"{"status":"success","data":[{"id":1,"username":"admin","password":"******"}]}"#.to_vec();
            } else {
                response.body = br#"{"status":"error","message":"No records found"}"#.to_vec();
            }
            response.content_type = "application/json".to_string();
        } else {
            response.body = br#"{"status":"ok"}"#.to_vec();
            response.content_type = "application/json".to_string();
        }

        response.body_string = String::from_utf8_lossy(&response.body).to_string();

        Ok(response)
    }
}

/// 从表达式中提取规则调用
fn extract_rule_calls(expression: &str) -> Vec<String> {
    let mut rule_calls = Vec::new();

    // 简单的规则提取逻辑，实际应用中应该使用更复杂的解析
    for part in expression.split(&['(', ')', '&', '|', '!', ' ']) {
        let part = part.trim();
        if !part.is_empty()
            && !part.contains(['+', '-', '*', '/', '.'])
            && !part.starts_with('"')
            && !part.starts_with('\'')
            && !part.eq("true")
            && !part.eq("false")
            && !part.eq("&&")
            && !part.eq("||")
            && !part.eq("!")
        {
            rule_calls.push(part.to_string());
        }
    }

    rule_calls
}
