use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    str,
};

use anyhow::Context;
use itertools::Itertools;

#[derive(Debug)]
enum Status {
    Ok,
    NotFound,
}

impl From<Status> for &[u8] {
    fn from(value: Status) -> Self {
        match value {
            Status::Ok => b"200 OK",
            Status::NotFound => b"404 Not Found",
        }
    }
}

#[derive(Debug)]
struct Response {
    status: Status,
    body: Option<Body>,
}

impl Response {
    fn with_status(status: Status) -> Self {
        Self { status, body: None }
    }

    fn from_status_and_body(status: Status, body: Body) -> Self {
        Self {
            status,
            body: Some(body),
        }
    }

    fn write(self, w: &mut impl Write) -> anyhow::Result<()> {
        w.write(b"HTTP/1.1 ").context("Writing proto")?;
        w.write(self.status.into()).context("Writing status")?;
        w.write(b"\r\n").context("end of start line")?;
        if let Some(body) = self.body {
            body.write(w).context("Writing body")?;
        } else {
            w.write(b"\r\n").context("end of start line")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Body {
    content_type: String,
    content: Vec<u8>,
}

impl Body {
    fn new(content_type: impl Into<String>, content: &[u8]) -> Self {
        Self {
            content: content.to_vec(),
            content_type: content_type.into(),
        }
    }

    fn write(self, w: &mut impl Write) -> anyhow::Result<()> {
        write!(w, "Content-Type: {} \r\n", self.content_type)?;
        write!(w, "Content-Length: {} \r\n", self.content.len())?;
        write!(w, "\r\n")?;
        w.write(&self.content)?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = Vec::new();
                let mut reader = BufReader::new(&mut stream);
                reader
                    .read_until(b'\r', &mut buf)
                    .context("Reading stream")?;

                let (_method, path, _proto) = buf
                    .splitn(3, |b| b.is_ascii_whitespace())
                    .collect_tuple()
                    .context("Should have 3 parts")?;
                let path = str::from_utf8(path).context("We only support utf8 for now")?;
                let path_parts = path.split("/").collect_vec();

                eprintln!("{path_parts:?}");

                let resp = match path_parts.get(1) {
                    Some(&"echo") => {
                        let rest = path_parts[2..].join("/");
                        let body = Body::new("text/plain", rest.as_bytes());
                        Response::from_status_and_body(Status::Ok, body)
                    }
                    Some(&"") => Response::with_status(Status::Ok),
                    _ => Response::with_status(Status::NotFound),
                };

                resp.write(&mut stream).context("Writing response")?;

                println!("received a connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}
