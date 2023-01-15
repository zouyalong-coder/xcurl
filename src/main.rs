use std::collections::HashMap;

use anyhow::Result;
use clap::Parser;
use reqwest::header::{self, HeaderMap};
use serde_json::json;
use xcurl::config::YamlConfigure;
use xcurl::KV;
use xcurl::{config::YamlConf, Cli, CurlArg, SubCommand};

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Cli::parse();
    match opts.subcmd {
        SubCommand::Curl(arg) => do_curl(arg).await,
    }
}

async fn do_curl(arg: CurlArg) -> Result<()> {
    // let conf = match arg.profile {
    //     Some(name) => {
    //         let path = resolve_conf_path(&name);
    //         YamlConf::load_yaml(&path).await?
    //     }
    //     None => YamlConf::empty(),
    // };
    let rt = CurlRuntime::new(arg);
    rt.run().await
}

#[inline]
fn resolve_conf_path(name: &str) -> String {
    const WORKSPACE: &str = "~/.xcurl";
    format!("{}/{}.yaml", WORKSPACE, name)
}

struct CurlRuntime {
    curl_arg: CurlArg,
}

impl CurlRuntime {
    pub fn new(curl_arg: CurlArg) -> Self {
        Self { curl_arg }
    }

    pub async fn run(&self) -> Result<()> {
        let (headers, query, body) = self.generate()?;
        let url = self.get_url();
        let client = reqwest::Client::new();
        let req = client
            .request(self.curl_arg.get_method(), url)
            .query(&query)
            .headers(headers)
            .body(body)
            .build()?;
        let resp = client.execute(req).await?;
        println!("{:?}", resp);
        Ok(())
    }

    fn get_content_type(&self) -> String {
        const DEFAULT_CT: &str = "application/json";
        self.curl_arg
            .get_headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| {
                v.to_str()
                    .unwrap()
                    .split(";")
                    .next()
                    .and_then(|v| Some(v.to_string()))
            })
            .unwrap_or(DEFAULT_CT.to_string())
    }

    /// 生成请求参数：请求头、query、body
    fn generate(&self) -> Result<(HeaderMap, Vec<(String, String)>, String)> {
        let headers = self.curl_arg.get_headers();
        let query = self
            .curl_arg
            .get_query()
            .iter()
            .map(|kv| (kv.key.clone(), kv.value.clone()))
            .collect();
        let body = self.curl_arg.get_body();
        let mut body_json = json!({});
        for kv in body {
            body_json[kv.key] = json!(kv.value);
        }
        let body = match self.get_content_type().as_str() {
            "application/json" => serde_json::to_string(&body_json)?,
            "application/x-www-form-urlencoded" | "multipart/form-data" => {
                serde_qs::to_string(&body_json)?
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "unsupported body encoding: {}",
                    self.get_content_type()
                ))
            }
        };
        Ok((headers, query, body))
    }

    fn get_url(&self) -> String {
        self.curl_arg.get_base_url().to_string()
    }
}
