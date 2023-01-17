use anyhow::Result;
use reqwest::{header::HeaderMap, Response};
use std::fmt::Write;
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

use crate::utils;

pub async fn get_body_text(response: Response) -> Result<(String, String)> {
    let headers = response.headers();
    let content_type = utils::get_content_type(headers);
    let body = response.text().await?;
    match content_type.as_str() {
        "application/json" => {
            let json = serde_json::from_str(&body)?;
            let json = serde_json::to_string_pretty(&json)?;
            Ok(("json".into(), json))
        }
        _ => match content_type.as_str() {
            "application/xml" => Ok(("xml".into(), body)),
            "text/html" => Ok(("html".into(), body)),
            _ => Ok(("txt".into(), body)),
        },
    }
}

lazy_static::lazy_static!(
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
);

pub fn highlight_text(text: &str, extension: &str, theme: Option<&str>) -> Result<String> {
    let syntax = if let Some(s) = SYNTAX_SET.find_syntax_by_extension(extension) {
        s
    } else {
        SYNTAX_SET.find_syntax_plain_text()
    };
    let mut h = HighlightLines::new(
        syntax,
        &THEME_SET.themes[theme.unwrap_or("base16-ocean.dark")],
    );
    let mut output = String::new();
    for line in LinesWithEndings::from(text) {
        let ranges = h.highlight_line(line, &SYNTAX_SET).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges, false);
        write!(&mut output, "{}", escaped)?;
    }
    Ok(output)
}

pub fn get_header_text_in_yaml(headers: &HeaderMap) -> String {
    let mut output = String::new();
    for (key, value) in headers {
        writeln!(&mut output, "{}: {}", key, value.to_str().unwrap()).unwrap();
    }
    output
}
