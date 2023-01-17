use super::{
    tools::{get_body_text, get_header_text_in_yaml, highlight_text},
    Output,
};
use anyhow::Result;
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
        let (extension, body) = get_body_text(response).await?;
        let mut stderr = String::new();
        let mut stdout = String::new();
        if self.stdout_color_mode {
            write!(
                stdout,
                "{}",
                highlight_text(body.as_str(), extension.as_str(), None)?
            )?;
        } else {
            write!(stdout, "{}", body)?;
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

    fn print_request(&self, request: &reqwest::Request) -> anyhow::Result<(String, String)> {
        Ok(("".into(), "".into()))
    }
}
