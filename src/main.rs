use std::fs::File;
use std::net::TcpListener;
use std::{path::Path, thread};

use anyhow::Result;
use async_native_tls::{Identity, TlsAcceptor};
use async_std::io::BufReader;
use futures::prelude::*;
use http_types::{Body, Request, Response, StatusCode};
use piper::{Arc, Mutex};
use smol::{blocking, reader, Async, Task};

/// Serves a request and returns a response.
async fn serve(req: Request) -> http_types::Result<Response> {
    // println!("Serving {}", req.url());

    let file_path = Path::new(".").join(&req.url().path()[1..]);
    // dbg!(&file_path);

    let file = blocking!(File::open(&file_path))?;
    // let file_len = blocking!(file.metadata())?.len();
    let file_reader = BufReader::new(Mutex::new(reader(file)));

    let body = Body::from_reader(Box::new(file_reader), None);

    let mut res = Response::new(StatusCode::Ok);
    // res.insert_header("Content-Type", "text/plain")?;
    res.set_body(body);
    Ok(res)
}

/// Listens for incoming connections and serves them.
async fn listen(listener: Async<TcpListener>, tls: Option<TlsAcceptor>) -> Result<()> {
    // Format the full host address.
    let host = match &tls {
        None => format!("http://{}", listener.get_ref().local_addr()?),
        Some(_) => format!("https://{}", listener.get_ref().local_addr()?),
    };
    println!("Listening on {}", host);

    loop {
        // Accept the next connection.
        let (stream, _) = listener.accept().await?;
        let host = host.clone();

        // Spawn a background task serving this connection.
        let task = match &tls {
            None => {
                let stream = Arc::new(stream);
                Task::spawn(async move {
                    if let Err(err) = async_h1::accept(&host, stream, serve).await {
                        println!("Connection error: {:#?}", err);
                    }
                })
            }
            Some(tls) => {
                // In case of HTTPS, establish a secure TLS connection first.
                match tls.accept(stream).await {
                    Ok(stream) => {
                        let stream = Arc::new(Mutex::new(stream));
                        Task::spawn(async move {
                            if let Err(err) = async_h1::accept(&host, stream, serve).await {
                                println!("Connection error: {:#?}", err);
                            }
                        })
                    }
                    Err(err) => {
                        println!("Failed to establish secure TLS connection: {:#?}", err);
                        continue;
                    }
                }
            }
        };

        // Detach the task to let it run in the background.
        task.detach();
    }
}

fn main() -> Result<()> {
    // Initialize TLS with the local certificate, private key, and password.
    // let identity = Identity::from_pkcs12(include_bytes!("identity.pfx"), "password")?;
    // let tls = TlsAcceptor::from(native_tls::TlsAcceptor::new(identity)?);

    // Create an executor thread pool.
    let num_threads = num_cpus::get().max(1);
    for _ in 0..num_threads {
        thread::spawn(|| smol::run(future::pending::<()>()));
    }

    // Start HTTP and HTTPS servers.
    smol::block_on(async {
        let http = listen(Async::<TcpListener>::bind("127.0.0.1:8000")?, None);
        http.await?;
        // let https = listen(Async::<TcpListener>::bind("127.0.0.1:8001")?, Some(tls));
        // future::try_join(http, https).await?;
        Ok(())
    })
}
