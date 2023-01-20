use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Result};
use clap::{ArgAction, Args, Parser, Subcommand};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method, Url,
};

#[derive(Parser)]
#[command(author, about, version)]
pub struct Cli {
    #[command(subcommand)]
    pub subcmd: SubCommand,
    /// verbose mode. Print headers in stderr.
    #[clap(short, long, action=ArgAction::SetTrue)]
    pub verbose: Option<bool>,
}

// ref to https://docs.rs/clap/latest/clap/_derive/_tutorial/
#[derive(Subcommand)]
pub enum SubCommand {
    /// do http request.
    Http(CurlArg),
    /// do HTTP Get. Same as `curl -M GET`.
    Get(CurlArg),
    /// do HTTP Post. Same as `curl -M POST`.
    Post(CurlArg),
    /// do HTTP Put. Same as `curl -M PUT`.
    Put(CurlArg),
    /// do HTTP Delete. Same as `curl -M DELETE`.
    Delete(CurlArg),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlPart {
    pub url: String,
    pub query: Vec<Param>,
}

#[derive(Args)]
pub struct CurlArg {
    /// url to request. Query params should be given by url.
    #[arg(value_parser=parse_url, value_name="URL & QUERY")]
    pub url_and_query: UrlPart,
    /// http method. Default is GET.
    #[clap(short, long)]
    pub method: Option<Method>,
    /// form data. Same as `curl -F`.
    #[arg(short='F', long, action=ArgAction::SetTrue, verbatim_doc_comment)]
    pub form: bool,
    /// multipart form data.
    #[arg(short='f', long, action=ArgAction::SetTrue)]
    pub multipart: bool,
    /// offline mode for request debug. Do not send request.
    #[arg(long, action=ArgAction::SetTrue)]
    pub offline: bool,
    /// Arguments for request.
    /// - `key:value` for header.
    /// - `key=value` for body.
    /// - `key==value` for query.
    #[arg(value_parser=parse_kv, value_name="HEADER & BODY", verbatim_doc_comment)]
    pub headers_and_body: Vec<Param>,
}

fn parse_url(s: &str) -> Result<UrlPart> {
    let s = if s.starts_with("http") {
        s.to_string()
    } else {
        "http://".to_string() + s
    };
    Url::parse(s.as_str())
        .map(|url| {
            let mut query = vec![];
            for (k, v) in url.query_pairs() {
                query.push(Param::Query(KV {
                    key: k.to_string(),
                    value: v.to_string(),
                }));
            }
            UrlPart {
                url: url.as_str().to_string(),
                query,
            }
        })
        .map_err(|e| anyhow!("invalid url: {}", e))
}

fn parse_kv(s: &str) -> Result<Param> {
    let mut iter = s.splitn(2, |c| c == ':' || c == '=');
    let key = iter.next().ok_or(anyhow!("invalid pair"))?;
    let value = iter.next().ok_or(anyhow!("invalid pair"))?;
    let key = key.trim();
    let value = value.trim();
    match s.chars().nth(key.len()) {
        Some(':') => Ok(Param::Header(KV {
            key: key.to_string(),
            value: value.to_string(),
        })),
        Some('=') => match value.chars().next() {
            // == is query
            Some('=') => Ok(Param::Query(KV {
                key: key.to_string(),
                value: value[1..].to_string(),
            })),
            _ => Ok(Param::Body(KV {
                key: key.to_string(),
                value: value.to_string(),
            })),
        },
        _ => Err(anyhow!("invalid pair")),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KV {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Param {
    Query(KV),
    Header(KV),
    Body(KV),
}

impl Param {
    pub fn is_query(&self) -> bool {
        matches!(*self, Self::Query(_))
    }

    pub fn is_header(&self) -> bool {
        matches!(*self, Self::Header(_))
    }

    pub fn is_body(&self) -> bool {
        matches!(*self, Self::Body(_))
    }
}

impl CurlArg {
    pub fn get_base_url(&self) -> &str {
        self.url_and_query.url.as_str()
    }

    pub fn get_method(&self) -> Method {
        let m = self.method.as_ref().unwrap_or(&Method::GET);
        m.clone()
    }

    pub fn get_query(&self) -> Vec<KV> {
        let mut query: Vec<KV> = self
            .url_and_query
            .query
            .iter()
            .map(|p| match p {
                Param::Query(kv) => kv.clone(),
                _ => panic!("invalid query"),
            })
            .collect();
        self.headers_and_body
            .iter()
            .filter_map(|p| match p {
                Param::Query(kv) => Some(kv),
                _ => None,
            })
            .for_each(|kv| {
                query.push(kv.clone());
            });
        query
    }

    pub fn get_headers(&self) -> HeaderMap {
        let mut hm: HeaderMap = self
            .headers_and_body
            .iter()
            .filter_map(|kv| match kv {
                Param::Header(kv) => {
                    let key = HeaderName::from_str(&kv.key).unwrap();
                    let value = HeaderValue::from_str(&kv.value).unwrap();
                    Some((key, value))
                }
                _ => None,
            })
            .collect();
        if self.form {
            hm.insert(
                HeaderName::from_static("content-type"),
                HeaderValue::from_static("application/x-www-form-urlencoded"),
            );
        } else if self.multipart {
            hm.insert(
                HeaderName::from_static("content-type"),
                HeaderValue::from_static("multipart/form-data"),
            );
        } else {
            hm.insert(
                HeaderName::from_static("content-type"),
                HeaderValue::from_static("application/json"),
            );
        }
        hm
    }

    pub fn get_body(&self) -> Vec<KV> {
        self.headers_and_body
            .iter()
            .filter_map(|kv| match *kv {
                Param::Body(ref kv) => Some(kv.clone()),
                _ => None,
            })
            .collect()
    }
}
