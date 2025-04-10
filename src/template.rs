use regex::Regex;
use std::collections::HashMap;

use cel_interpreter::Value;

/// 渲染包含模板变量的字符串
///
/// # 参数
///
/// * `template` - 包含模板变量的字符串，如 "Hello, {{name}}!"
/// * `variables` - 变量映射，如 {"name": "World"}
///
/// # 返回
///
/// 渲染后的字符串，如 "Hello, World!"
pub fn render_template(template: &str, variables: &HashMap<String, Value>) -> String {
    // 创建正则表达式来匹配 {{var}} 格式的模板变量
    let re = Regex::new(r"\{\{([^}]+)\}\}").unwrap();

    // 替换所有匹配的模板变量
    re.replace_all(template, |caps: &regex::Captures| {
        let var_name = &caps[1];
        if let Some(value) = variables.get(var_name) {
            match value {
                Value::String(s) => s.to_string(),
                Value::Int(i) => i.to_string(),
                Value::Float(f) => f.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => format!("{:?}", value),
            }
        } else {
            // 如果变量不存在，保留原始模板变量
            caps[0].to_string()
        }
    })
    .to_string()
}

/// 渲染请求中的模板变量
///
/// # 参数
///
/// * `request` - 请求对象
/// * `variables` - 变量映射
pub fn render_request(request: &mut crate::poc::Request, variables: &HashMap<String, Value>) {
    // 渲染路径
    request.path = render_template(&request.path, variables);

    // 渲染请求体
    request.body = render_template(&request.body, variables);

    // 渲染请求头
    let headers = std::mem::take(&mut request.headers);
    for (name, value) in headers {
        let rendered_name = render_template(&name, variables);
        let rendered_value = render_template(&value, variables);
        request.headers.insert(rendered_name, rendered_value);
    }
}

/// 渲染详情字段中的模板变量
///
/// # 参数
///
/// * `detail` - 详情对象
/// * `variables` - 变量映射
pub fn render_detail(detail: &mut crate::poc::Detail, variables: &HashMap<String, Value>) {
    // 渲染漏洞匹配字段
    if !detail.vulnerability.r#match.is_empty() {
        detail.vulnerability.r#match = render_template(&detail.vulnerability.r#match, variables);
    }

    // 渲染指纹版本字段
    if !detail.fingerprint.version.is_empty() {
        detail.fingerprint.version = render_template(&detail.fingerprint.version, variables);
    }

    // 渲染其他自定义字段
    let custom_fields = std::mem::take(&mut detail.vulnerability.custom_fields);
    for (name, value) in custom_fields {
        let rendered_value = render_template(&value, variables);
        detail
            .vulnerability
            .custom_fields
            .insert(name, rendered_value);
    }
}
