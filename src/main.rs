use std::env;

use clap::Parser;
use reqwest::header::HeaderMap;
use reqwest::Method;
use serde_json::json;
use std::io::Write;
use xcurl::error::Result;
use xcurl::output::{select_output, Output};
use xcurl::utils::get_content_type;
use xcurl::{Cli, CurlArg, SubCommand};

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    let opts = Cli::parse();
    match opts.subcmd {
        SubCommand::Http(arg) => do_curl(arg).await,
        SubCommand::Get(mut arg) => {
            arg.method = Some(Method::GET);
            do_curl(arg).await
        }
        SubCommand::Post(mut arg) => {
            arg.method = Some(Method::POST);
            do_curl(arg).await
        }
        SubCommand::Put(mut arg) => {
            arg.method = Some(Method::PUT);
            do_curl(arg).await
        }
        SubCommand::Delete(mut arg) => {
            arg.method = Some(Method::DELETE);
            do_curl(arg).await
        }
    }
}

async fn do_curl(arg: CurlArg) -> Result<()> {
    let rt = CurlRuntime::new(arg);
    rt.run().await
}

struct CurlRuntime {
    curl_arg: CurlArg,
    output: Box<dyn Output>,
}

impl CurlRuntime {
    pub fn new(curl_arg: CurlArg) -> Self {
        let output = Box::new(select_output());
        Self { curl_arg, output }
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

        let url = req.url().to_string();

        if self.curl_arg.offline {
            let (out, err) = self.output.print_request(&req)?;
            self.print(url.as_str(), out, err).await?;
        } else {
            let resp = client.execute(req).await?;
            let (out, err) = self.output.print_response(resp).await?;
            self.print(url.as_str(), out, err).await?;
        }

        Ok(())
    }

    async fn print(&self, url: &str, out: String, err: String) -> Result<()> {
        {
            let stderr = std::io::stderr();
            let mut stderr = stderr.lock();
            writeln!(stderr, "{}", url)?;
            write!(stderr, "{}", err)?;
        }

        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        write!(stdout, "{}", out)?;
        Ok(())
    }

    fn get_content_type(&self) -> String {
        get_content_type(&self.curl_arg.get_headers())
    }

    /// 生成请求参数：请求头、query、body
    fn generate(&self) -> Result<(HeaderMap, Vec<(String, String)>, String)> {
        let headers = self.curl_arg.get_headers();
        let query = self
            .curl_arg
            .get_query()
            .iter()
            .map(|kv| (kv.key.clone(), kv.value.value().to_string()))
            .collect();
        let body = self.curl_arg.get_body();
        let mut body_json = json!({});
        for kv in body {
            body_json[kv.key] = json!(kv.value.value().to_string());
        }
        let body = match self.get_content_type().as_str() {
            "application/json" => serde_json::to_string(&body_json)?,
            "application/x-www-form-urlencoded" | "multipart/form-data" => {
                serde_qs::to_string(&body_json).map_err(|x| anyhow::anyhow!(x))?
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "unsupported body encoding: {}",
                    self.get_content_type()
                )
                .into())
            }
        };
        Ok((headers, query, body))
    }

    fn get_url(&self) -> String {
        self.curl_arg.get_base_url().to_string()
    }
}
