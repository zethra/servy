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

use clap::ArgMatches;
use futures::future::FutureResult;
use hyper::StatusCode;
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
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, request: Request) -> Self::Future {
        if MATCHES.is_present("verbose") {
            println!("{} {}", request.method(), request.path());
        }
        let path_str = ".".to_string() + request.path();
        let path = Path::new(&path_str);
        futures::future::ok(serve_file(path))
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
    Response::new().with_body(content)
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
    let mut server = Http::new().bind(&addr, || Ok(Servy)).unwrap();
    server.no_proto();
    println!("Starting server on http://{}", server.local_addr().unwrap());
    server.run().unwrap();
}
