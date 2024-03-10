use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
};

use anyhow::Context;
use itertools::Itertools;

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

                match path {
                    [b'/'] => stream.write(b"HTTP/1.1 200 OK\r\n\r\n")?,
                    _ => stream.write(b"HTTP/1.1 404 OK\r\n\r\n")?,
                };
                println!("received a connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}
