use crate::{
    request::Request,
    response::{Body, Response, Status},
};
use anyhow::Context;
use clap::Parser;
use itertools::Itertools;
use std::{
    fs::{self, File},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

mod request;
mod response;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "/tmp")]
    directory: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || handle_request(stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_request(mut stream: TcpStream) -> anyhow::Result<()> {
    let args = Args::parse();
    let req = Request::read(&mut stream).context("Parsing request")?;

    let path_parts = req.path.split("/").collect_vec();

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
        Some(&"files") => {
            let filename = path_parts.get(2).context("Should provide a filename")?;
            let dir: PathBuf = args.directory;
            let path = dir.join(filename);
            let file = fs::read(path)?;
            Response::from_status_and_body(Status::Ok, Body::new("text/plain", &file))
        }
        Some(&"") => Response::with_status(Status::Ok),
        _ => Response::with_status(Status::NotFound),
    };

    resp.write(&mut stream).context("Writing response")?;

    println!("received a connection");
    Ok(())
}
