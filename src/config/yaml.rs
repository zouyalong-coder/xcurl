//! 单个yaml 文件配置
use anyhow::Result;
use reqwest::Method;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

use super::validate::Validator;
// use void::Void;

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlConf {
    /// 通用配置
    pub common: Common,
    // #[serde(flatten)] // 类似于inline
    pub requests: HashMap<String, Request>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Common {
    #[serde(rename = "query", default)]
    pub default_query: Option<serde_json::Value>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    #[serde(with = "http_serde::method", default)]
    pub method: Method,
    pub url: String,
    pub headers: HashMap<String, String>,
    // #[serde(serialize_with = "")]
    pub query: Option<serde_json::Value>,
    pub body: Option<serde_json::Value>,
}

impl YamlConf {
    pub fn empty() -> Self {
        Self {
            common: Common {
                headers: HashMap::new(),
                default_query: None,
            },
            requests: HashMap::new(),
        }
    }

    pub fn to_yaml(&self) -> Result<String> {
        let content = serde_yaml::to_string(self)?;
        Ok(content)
    }
}

impl Validator for YamlConf {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

impl YamlConfigure for YamlConf {}

#[async_trait::async_trait]
pub trait YamlConfigure
where
    Self: Sized + DeserializeOwned + Validator,
{
    async fn load_yaml(path: &str) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_string(&content)
    }
    fn from_string(content: &str) -> Result<Self> {
        let conf: Self = serde_yaml::from_str(content)?;
        conf.validate()?;
        Ok(conf)
    }
}
