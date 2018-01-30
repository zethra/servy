#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate futures;
extern crate hyper;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;
use std::fs::metadata;
use std::thread;

use clap::ArgMatches;
use futures::future::ok;
use futures::future::Future;
use futures::sync::{mpsc, oneshot};
use futures::sink::Sink;
use hyper::{Chunk, StatusCode};
use hyper::header::ContentLength;
use hyper::server::{Http, Service, Request, Response};

lazy_static! {
    static ref MATCHES: ArgMatches<'static> = clap_app!(servy =>
            (version: "0.1.0")
            (author: "Ben Goldberg <jediben97@gmail.com>")
            (about: "A tiny little web server")
            (@arg verbose: -v --verbose "Verbose output")
            (@arg host: -h --host +takes_value "Host string the web server should use ie. 0.0.0.0")
            (@arg port: -p --port +takes_value "The port web server should use ie. 8000")
        ).get_matches();
}

struct Servy;

impl Service for Servy {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, request: Request) -> Self::Future {
        if MATCHES.is_present("verbose") {
            println!("{} {}", request.method(), request.path());
        }
        let path_str = ".".to_string() + request.path();
        let path = Path::new(path_str.as_str()).to_owned();
        let data = match metadata(&path) { 
            Ok(data) => data,
            Err(_) => {
                return Box::new(ok(Response::new()
                           .with_status(StatusCode::NotFound)
                           .with_body("File not found")));
            }
        };
        if data.len() < 1000000 {
            Box::new(ok(serve_file(&path)))
        } else { 
            let (tx, rx) = oneshot::channel();
            thread::spawn(move || {
                let mut file = match File::open(path) {
                    Ok(file) => file,
                    Err(_) => {
                        tx.send(Response::new()
                            .with_status(StatusCode::NotFound)
                            .with_body("File not found"))
                        .expect("Send error on open");
                        return;
                    }
                };
                let (mut tx_body, rx_body) = mpsc::channel(1);
                let resp = Response::new()
                    .with_header(ContentLength(data.len()))
                    .with_body(rx_body);
                tx.send(resp).expect("Send error on successful file read");
                let mut buf = [0u8; 16];
                loop {
                    match file.read(&mut buf) {
                        Ok(n) => {
                            if n == 0 {
                                tx_body.close().expect("panic closing");
                                break;
                            } else {
                                let chunk: Chunk = buf[0..n].to_vec().into();
                                match tx_body.send(Ok(chunk)).wait() {
                                    Ok(t) => { tx_body = t }
                                    Err(_) => { 
                                        break;
                                    }
                                };
                            }
                        },
                        Err(_) => { 
                            break;
                        }
                    }
                }
            });

            Box::new(rx.map_err(|e| hyper::Error::from(std::io::Error::new(std::io::ErrorKind::Other, e))))
        }
    }
}

fn serve_file(path: &Path) -> Response {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => {
            return Response::new()
                .with_status(StatusCode::NotFound)
                .with_body("File not found");
        }
    };
    let mut buf_reader = BufReader::new(file);
    let mut content = Vec::new();
    if let Err(e) = buf_reader.read_to_end(&mut content) {
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
    <hr/>
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
                                if name_str == "index.html" || name_str == "index.htm" {
                                    return serve_file(path.as_path());
                                }
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
    <hr/>
</body>
</html>
                    "#);
                    return Response::new()
                        .with_header(ContentLength(page.as_bytes().len() as u64))
                        .with_body(page);
                }
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
        println!("{}", e);
        return Response::new()
            .with_status(StatusCode::InternalServerError)
            .with_body("Internal error");
    }
    Response::new()
        .with_header(ContentLength(content.len() as u64))
        .with_body(content)
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
    let server = Http::new().bind(&addr, || Ok(Servy)).unwrap();
    println!("Starting server on http://{}", server.local_addr().unwrap());
    server.run().unwrap();
}
