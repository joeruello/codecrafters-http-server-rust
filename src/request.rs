use std::{
    collections::HashMap,
    io::Read,
    str::{self, FromStr},
};

use anyhow::{anyhow, Context};
use itertools::Itertools;

pub(crate) enum Method {
    Get,
    Post,
}

impl FromStr for Method {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            _ => Err(anyhow!("Unknown method")),
        }
    }
}

pub(crate) struct Request {
    pub(crate) method: Method,
    pub(crate) path: String,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) content: Option<Vec<u8>>,
}

impl Request {
    pub(crate) fn read(r: &mut impl Read) -> anyhow::Result<Self> {
        let mut buf = [0; 1024];
        let n = r.read(&mut buf).context("Reading stream")?;

        let buf = str::from_utf8(&buf[..n]).context("Only utf8")?;
        let parts = buf.split("\r\n\r\n").collect_vec();
        let mut lines = parts.first().context("Should have a header")?.lines();
        let (method, path, _proto) = lines
            .next()
            .context("should have a header")?
            .splitn(3, |c: char| c.is_whitespace())
            .collect_tuple()
            .context("Should have 3 parts")?;

        let headers: HashMap<String, String> = lines
            .filter_map(|l| match l.split_once(": ") {
                Some((k, v)) => Some((k.to_lowercase(), v.to_string())),
                None => None,
            })
            .collect();

        Ok(Self {
            method: method.parse().context("Parse method")?,
            path: path.into(),
            headers,
            content: parts.get(1).map(|p| p.as_bytes().to_vec()),
        })
    }
}
