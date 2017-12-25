extern crate futures;
extern crate tokio_minihttp;
extern crate tokio_proto;
extern crate tokio_service;

use std::io;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use futures::future;
use tokio_minihttp::{Request, Response, Http};
use tokio_proto::TcpServer;
use tokio_service::Service;

struct HelloWorld;

impl Service for HelloWorld {
    type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Future = future::Ok<Response, io::Error>;

    fn call(&self, request: Request) -> Self::Future {
        let mut resp = Response::new();
        let file = match File::open(".".to_string() + request.path()) {
            Ok(file) => file,
            Err(_) => {
                resp.status_code(404, "File not found");
                resp.body("File not found");
                return future::ok(resp);
            }
        };
        let mut buf_reader = BufReader::new(file);
        let mut content = Vec::new();
        if let Err(e) = buf_reader.read_to_end(&mut content) {
            resp.status_code(500, "Internal error");
            resp.body("Internal error");
            println!("{}", e);
            return future::ok(resp);
        }
        resp.body_bytes(content.as_slice());
        future::ok(resp)
    }
}

fn main() {
    let addr = "[::1]:8000".parse().unwrap();
    println!("Starting server on http://[::1]:8000");
    TcpServer::new(Http, addr)
        .serve(|| Ok(HelloWorld));
}
