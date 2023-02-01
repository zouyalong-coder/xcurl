mod colorful_tty;
mod tools;

use crate::error::Result;

#[async_trait::async_trait]
pub trait Output {
    async fn print_response(&self, response: reqwest::Response) -> Result<(String, String)>;
    fn print_request(&self, request: &reqwest::Request) -> Result<(String, String)>;
}

pub fn select_output() -> impl Output {
    let stdout_color_mode = atty::is(atty::Stream::Stdout);
    let stderr_color_mode = atty::is(atty::Stream::Stderr);
    colorful_tty::ColorfulTTY {
        stdout_color_mode,
        stderr_color_mode,
    }
}
