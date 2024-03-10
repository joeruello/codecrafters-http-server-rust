use crate::{
    request::Request,
    response::{Body, Response, Status},
};
use anyhow::Context;
use itertools::Itertools;
use std::net::TcpListener;

mod request;
mod response;

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let req = Request::read(&mut stream).context("Parsing request")?;

                let path_parts = req.path.split("/").collect_vec();

                eprintln!("{path_parts:?}");

                let resp = match path_parts.get(1) {
                    Some(&"echo") => {
                        let rest = path_parts[2..].join("/");
                        let body = Body::new("text/plain", rest.as_bytes());
                        Response::from_status_and_body(Status::Ok, body)
                    }
                    Some(&"user-agent") => {
                        let body = Body::new(
                            "text/plain",
                            req.headers
                                .get("user-agent")
                                .context("Should have a user-agent")?
                                .as_bytes(),
                        );
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
