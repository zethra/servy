# Servy

#### A tiny little web server

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