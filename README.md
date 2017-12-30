# Servy

#### A tiny little web server

[![Build Status](https://www.travis-ci.org/zethra/servy.svg?branch=master)](https://www.travis-ci.org/zethra/servy)

## What is servy?

Well it's a little tiny web server written in rust.  
It's a single binary with only a few flag meant to be analogous to `python -m http.server`

### Important Note
By default servy starts on the ipv6 interface by using the host string `[::1]`
If you want to start it on the ipv4 interface use a ipv4 host string (see below)

## Usage 

`servy` 

Start a web server on port 8000

`servy -p 8080`

Start servy on port 8080

`servy -h 127.0.0.1`

Start servy with host string 127.0.0.1 (start on the ipv4 loopback interface)

`servy -v`

Use verbose output (print out a debug message every time a connection is made)

`servy --help`

Print the help message

## Stuff inside

Servy in built on top of tokio-minihttp.  I like it because it's small and simple, like servy.
It is currently using a fork of tokio-minihttp because it stores the responses as strings, and
as rust requires all strings to be valid UTF-8 I could only server text files.  I forked tokio-minihttp
and modified it to store responses as bytes and am waiting on my pull request to be merged (fingers crossed).

## Benchmarks (cuz why not)

As servy is built on tokio-minihttp it's quite fast.
On my laptop with an Intel Core i7-4700MQ @ 2.4GHz with 16GB of RAM
When I run `wrk -c 200 -d 10 -t 20 http://\[::1\]:8000/Cargo.toml`
I get
```
Running 10s test @ http://[::1]:8000/Cargo.toml
  20 threads and 200 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.56ms  209.60us   5.71ms   89.90%
    Req/Sec     6.46k     1.23k   28.10k    99.06%
  1297370 requests in 10.10s, 540.69MB read
Requests/sec: 128459.00
Transfer/sec:     53.54MB
```

For comparision running the same test on `python -m http.server` on Python 3.6.4
```
Running 10s test @ http://0.0.0.0:8000/Cargo.toml
  20 threads and 200 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    10.13ms   72.55ms   1.68s    97.76%
    Req/Sec   371.84    314.60     1.94k    76.29%
  20260 requests in 10.08s, 10.55MB read
  Socket errors: connect 0, read 0, write 0, timeout 17
Requests/sec:   2009.57
Transfer/sec:      1.05MB
```

## Contributing

If you run into any issues, have any suggestions on improvements, on even a pull request, shot it my way.
All feedback is welcome and appreciated!