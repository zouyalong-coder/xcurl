use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method, Url,
};

#[derive(Parser)]
#[command(author, about, version)]
pub struct Cli {
    #[command(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Subcommand)]
pub enum SubCommand {
    /// do curl.
    Curl(CurlArg),
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
    // configure
    // request profile in yaml.
    // #[clap(short, long)]
    // pub profile: Option<String>,
    /// verbose mode. Print headers in stderr.
    #[clap(short, long, default_value = "true")]
    pub verbose: bool,

    /// url to request. Query params could be given by url.
    #[clap(short, long, value_parser=parse_url)]
    pub url: UrlPart,
    /// http method. Default is GET.
    #[clap(short, long)]
    pub method: Option<Method>,
    /// Overrides args. Could be used to override the query, headers and body of the request.
    /// For query params, use `-e key=value`.
    /// For headers, use `-e %key=value`.
    /// For body, use `-e @key=value`.
    #[clap(short, value_parser=parse_kv)]
    pub extra_params: Vec<Param>,
}

fn parse_url(s: &str) -> Result<UrlPart> {
    Url::parse(s)
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
    let mut iter = s.splitn(2, '=');
    let key = iter.next().ok_or_else(|| anyhow!("invalid key"))?.trim();
    let value = iter.next().ok_or_else(|| anyhow!("invalid value"))?.trim();
    match key.chars().next() {
        Some('%') => Ok(Param::Header(KV {
            key: key[1..].to_string(),
            value: value.to_string(),
        })),
        Some('@') => Ok(Param::Body(KV {
            key: key[1..].to_string(),
            value: value.to_string(),
        })),
        Some(v) if v.is_alphabetic() => Ok(Param::Query(KV {
            key: key.to_string(),
            value: value.to_string(),
        })),
        _ => Err(anyhow!("invalid key")),
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

impl CurlArg {
    pub fn get_base_url(&self) -> &str {
        self.url.url.as_str()
    }

    pub fn get_method(&self) -> Method {
        let m = self.method.as_ref().unwrap_or(&Method::GET);
        m.clone()
    }

    pub fn get_query(&self) -> Vec<KV> {
        let mut query_map = HashMap::new();
        self.url.query.iter().for_each(|p| match p {
            Param::Query(kv) => {
                query_map.insert(kv.key.as_str(), kv);
            }
            _ => {}
        });
        let extra_query: Vec<Param> = self
            .extra_params
            .iter()
            .filter(|&p| match *p {
                Param::Query(_) => true,
                _ => false,
            })
            .map(|p| p.clone())
            .collect();
        extra_query.iter().for_each(|p| match p {
            Param::Query(kv) => {
                query_map.insert(kv.key.as_str(), kv);
            }
            _ => {}
        });
        query_map.values().map(|&kv| kv.clone()).collect()
    }

    pub fn get_headers(&self) -> HeaderMap {
        self.extra_params
            .iter()
            .filter_map(|kv| match kv {
                Param::Header(kv) => {
                    let key = HeaderName::from_str(&kv.key).unwrap();
                    let value = HeaderValue::from_str(&kv.value).unwrap();
                    Some((key, value))
                }
                _ => None,
            })
            .collect()
    }

    pub fn get_body(&self) -> Vec<KV> {
        self.extra_params
            .iter()
            .filter_map(|kv| match *kv {
                Param::Body(ref kv) => Some(kv.clone()),
                _ => None,
            })
            .collect()
    }
}
