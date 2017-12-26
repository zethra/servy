#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate futures;
extern crate tokio_minihttp;
extern crate tokio_proto;
extern crate tokio_service;

use std::io;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;

use clap::ArgMatches;
use futures::future;
use tokio_minihttp::{Request, Response, Http};
use tokio_proto::TcpServer;
use tokio_service::Service;

lazy_static! {
    static ref MATCHES: ArgMatches<'static> = clap_app!(servy =>
            (version: "0.1.0")
            (author: "Ben Goldberg <benaagoldberg@gmail.com>")
            (about: "A tiny file server")
            (@arg verbose: -v --verbose "Verbose output")
            (@arg host: -h --host +takes_value "Host string the web server should use ie. 0.0.0.0")
            (@arg port: -p --port +takes_value "The port web server should use ie. 8000")
        ).get_matches();
}

struct Server;

impl Service for Server {
    type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Future = future::Ok<Response, io::Error>;

    fn call(&self, request: Request) -> Self::Future {
        if MATCHES.is_present("verbose") {
            println!("{} {}", request.method(), request.path());
        }
        let mut resp = Response::new();
        let path_str = ".".to_string() + request.path();
        let path = Path::new(&path_str);
        if path.is_dir() {
            match path.read_dir() {
                Ok(dir) => {
                    let mut page = String::new();
                    page.push_str(r#"
<html>
<head>
    <title>Directory listing</title>
</head>
<body>
    <h1>Directory listing</h1>
    <ul>
"#);
                    for item in dir {
                        match item {
                            Ok(item) => {
                                let path = item.path();
                                let mut path_str = path.to_string_lossy().to_string();
                                path_str.remove(0);
                                let name = item.file_name();
                                let mut name_str = name.to_string_lossy().to_string();
                                if path.is_dir() {
                                    name_str.push('/');
                                }
                                page.push_str(format!("\t\t<li><a href=\"{}\">{}</a></li>\n", path_str, name_str).as_str());
                            }
                            Err(e) => {
                                println!("{}", e);
                            }
                        }
                    }
                    page.push_str(r#"
    </ul>
</body>
</html>
                    "#);
                    resp.body(page.as_str());
                }
                Err(e) => {
                    println!("{}", e);
                }
            }
        } else {
            let file = match File::open(path) {
                Ok(file) => file,
                Err(_) => {
                    resp.status_code(404, "File not found");
                    resp.body("File norust check it found");
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
        }
        future::ok(resp)
    }
}

fn main() {
    let host = match MATCHES.value_of("host") {
        Some(value) => value,
        None => "[::1]"
    };
    let port = match MATCHES.value_of("port") {
        Some(value) => value,
        None => "8000"
    };
    let addr_str = host.to_string() + ":" + port;
    let addr = match addr_str.as_str().parse() {
        Ok(addr) => addr,
        Err(_) => {
            println!("Invalid host or port");
            return;
        }
    };
    println!("Starting server on http://{}", addr_str);
    TcpServer::new(Http, addr)
        .serve(|| Ok(Server));
}
