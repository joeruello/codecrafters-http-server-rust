use std::{collections::HashMap, io::Read, str};

use anyhow::Context;
use itertools::Itertools;

pub(crate) struct Request {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) proto: String,
    pub(crate) headers: HashMap<String, String>,
}

impl Request {
    pub(crate) fn read(r: &mut impl Read) -> anyhow::Result<Self> {
        let mut buf = [0; 1024];
        r.read(&mut buf).context("Reading stream")?;

        let buf = str::from_utf8(&buf).context("Only utf8")?;
        let mut lines = buf.lines();
        let (method, path, proto) = lines
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
            method: method.into(),
            path: path.into(),
            proto: proto.into(),
            headers,
        })
    }
}
