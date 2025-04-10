use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 请求结构体，对应 xray v2 中的 request 部分
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Request {
    /// 请求方法 (GET, POST, PUT, DELETE 等)
    pub method: String,
    /// 是否缓存请求
    #[serde(default = "default_true")]
    pub cache: bool,
    /// 是否跟随重定向
    #[serde(default = "default_false")]
    pub follow_redirects: bool,
    /// 请求路径，包括 querystring
    pub path: String,
    /// 请求头
    pub headers: HashMap<String, String>,
    /// 请求体
    pub body: String,
    /// 请求超时时间(单位：秒)
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_timeout() -> u64 {
    10
}

/// 规则结构体，对应 xray v2 中的 rule
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Rule {
    /// 请求
    pub request: Request,
    /// 表达式
    pub expression: String,
    /// 从输出提取变量
    pub output: HashMap<String, String>,
}

/// 传输方式枚举，对应 xray v2 中的 transport
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Transport {
    #[serde(rename = "HTTP")]
    #[serde(alias = "http")]
    HTTP,
    #[serde(rename = "TCP")]
    #[serde(alias = "tcp")]
    TCP,
    #[serde(rename = "UDP")]
    #[serde(alias = "udp")]
    UDP,
}

impl Default for Transport {
    fn default() -> Self {
        Self::HTTP
    }
}

/// 指纹结构体，对应 xray v2 中的 fingerprint
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Fingerprint {
    /// 指纹库id，一般自动生成，可以不用填写
    pub id: String,
    /// 通用名称，一般自动生成，可以不用填写
    pub name: String,
    /// 用来接收指纹插件匹配到的版本信息，一般直接用这样的固定格式即可，会自动将output中匹配到的内容渲染过来
    pub version: String,
    /// 输出该指纹对应的产品的cpe，一般自动生成，可以不用填写
    pub cpe: String,
    /// 其他自定义输出字段
    #[serde(flatten)]
    pub custom_fields: HashMap<String, String>,
}

/// 漏洞结构体，对应 xray v2 中的 vulnerability
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Vulnerability {
    /// 漏洞库id，一般自动生成，可以不用填写
    pub id: String,
    /// 漏洞危害等级，一般自动根据漏洞信息生成，可以不用填写
    pub level: String,
    /// 一些证明漏洞存在的信息，比如信息泄露泄露的一些敏感数据等
    pub r#match: String,
    /// 其他自定义输出字段
    #[serde(flatten)]
    pub custom_fields: HashMap<String, String>,
}

/// 详情结构体，对应 xray v2 中的 detail
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Detail {
    /// 作者（个人主页）
    pub author: String,
    /// 参考链接
    pub links: Vec<String>,
    /// 一些警告信息，也就是该poc可能会产生的问题，比如产生脏数据等
    pub warning: String,
    /// 对该poc/漏洞的描述
    pub description: String,
    /// 当该插件为指纹插件时才需要填写，且一般会自动生成
    pub fingerprint: Fingerprint,
    /// 当该插件为漏洞插件且需要输出一些内容时才需要填写，且一般会自动生成
    pub vulnerability: Vulnerability,
}

/// Payload 结构体，对应 xray v2 中的 payloads
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Payloads {
    /// 命中一个之后是否继续
    pub continue_: bool,
    /// 多个 payload 的集合
    pub payloads: HashMap<String, HashMap<String, String>>,
}

/// POC 结构体，对应 xray v2 中的整个 YAML 文件
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Poc {
    /// POC 名称
    pub name: String,
    /// 是否手动，手动时不会自动执行
    #[serde(default)]
    pub manual: bool,
    /// 协议, 默认 http
    pub transport: Transport,
    /// 变量集合
    pub set: HashMap<String, serde_yaml::Value>,
    /// 规则
    pub rules: HashMap<String, Rule>,
    /// 表达式，用于组织 rule 的执行逻辑
    pub expression: String,
    /// 详情
    pub detail: Detail,
    /// 多个 payload 的配置
    pub payloads: Payloads,
}
