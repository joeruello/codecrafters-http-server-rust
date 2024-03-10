use anyhow::Context;
use std::io::Write;

#[derive(Debug)]
pub(crate) enum Status {
    Ok,
    Created,
    NotFound,
    BadRequest,
}

impl From<Status> for &[u8] {
    fn from(value: Status) -> Self {
        match value {
            Status::Ok => b"200 OK",
            Status::NotFound => b"404 Not Found",
            Status::Created => b"201 Created",
            Status::BadRequest => b"401 Bad Request",
        }
    }
}

#[derive(Debug)]
pub(crate) struct Response {
    status: Status,
    body: Option<Body>,
}

impl Response {
    pub(crate) fn with_status(status: Status) -> Self {
        Self { status, body: None }
    }

    pub(crate) fn from_status_and_body(status: Status, body: Body) -> Self {
        Self {
            status,
            body: Some(body),
        }
    }

    pub(crate) fn write(self, w: &mut impl Write) -> anyhow::Result<()> {
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
pub(crate) struct Body {
    content_type: String,
    content: Vec<u8>,
}

impl Body {
    pub(crate) fn new(content_type: impl Into<String>, content: &[u8]) -> Self {
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
