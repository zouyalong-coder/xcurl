use reqwest::header::{self, HeaderMap};

pub fn get_content_type(headers: &HeaderMap) -> String {
    const DEFAULT_CT: &str = "application/json";
    headers
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
