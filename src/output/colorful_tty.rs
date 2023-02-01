use crate::utils;

use super::{
    tools::{format_body, get_header_text_in_yaml, highlight_text},
    Output,
};
use crate::error::Result;
use colored::Colorize;
use std::fmt::Write;

pub struct ColorfulTTY {
    pub stdout_color_mode: bool,
    pub stderr_color_mode: bool,
}

#[async_trait::async_trait]
impl Output for ColorfulTTY {
    async fn print_response(&self, response: reqwest::Response) -> Result<(String, String)> {
        let head_line = format!("{:?} {}", response.version(), response.status(),);
        let header_text = get_header_text_in_yaml(response.headers());
        let headers = response.headers();
        let content_type = utils::get_content_type(headers);
        let body = {
            let body = response.text().await?;
            if body.is_empty() {
                None
            } else {
                Some(body)
            }
        };
        println!("body: {:?}", body);
        let (extension, body) = format_body(content_type.as_str(), body)?;

        let mut stderr = String::new();
        let mut stdout = String::new();
        if let Some(body) = body {
            if self.stdout_color_mode {
                write!(
                    stdout,
                    "{}",
                    highlight_text(body.as_str(), extension.as_str(), None)?
                )?;
            } else {
                write!(stdout, "{}", body)?;
            }
        }
        writeln!(stderr, "{}", head_line)?;
        if self.stderr_color_mode {
            write!(
                stderr,
                "{}",
                highlight_text(header_text.as_str(), "yaml", None)?
            )?;
        } else {
            write!(stderr, "{}", header_text)?;
        }
        Ok((stdout, stderr))
    }

    fn print_request(&self, request: &reqwest::Request) -> Result<(String, String)> {
        let mut stderr = String::new();
        let method = request.method().as_str();
        let path = request.url().path();
        let version = format!("{:?}", request.version());
        let head_line = if self.stderr_color_mode {
            format!("{} {} {}", method.yellow(), path.white(), version.green(),)
        } else {
            format!("{} {} {}", method, path, version,)
        };
        writeln!(stderr, "{}", head_line)?;
        let header_text = get_header_text_in_yaml(request.headers());
        let headers = request.headers();
        let content_type = utils::get_content_type(headers);
        let body = request.body().and_then(|v| {
            let v = v.as_bytes().unwrap();
            let v = String::from_utf8_lossy(v);
            Some(v.to_string())
        });
        let (extension, body) = format_body(content_type.as_str(), body)?;
        // let (extension, body) = match body {
        //     Some(body) => {
        //         let (ext, b) = format_body(content_type.as_str(), body)?;
        //         (ext, Some(b))
        //     }
        //     None => ("txt".to_string(), None),
        // };
        if self.stderr_color_mode {
            write!(
                stderr,
                "{}",
                highlight_text(header_text.as_str(), "yaml", None)?
            )?;
            if let Some(body) = body {
                write!(
                    stderr,
                    "{}",
                    highlight_text(body.as_str(), extension.as_str(), None)?
                )?;
            }
        } else {
            write!(stderr, "{}", header_text)?;
            if let Some(body) = body {
                write!(stderr, "{}", body)?;
            }
        }
        Ok(("".to_string(), stderr))

        // let (extension, body) = get_body_text(response).await?;
    }
}
