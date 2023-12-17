use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    const ADDR: &'static str = "127.0.0.1";
    const PORT: usize = 1000;
    let url = format!("{ADDR}:{PORT}");

    let listener = TcpListener::bind(&url).unwrap();
    println!("Server started on: {:?}", &url);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
        println!("Connection established!");
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    let request: String = http_request[1].to_string();

    println!("Request: {:#?}", http_request);
    if (request == "GET / HTTP/1.1") {
        let status = "HTTP/1.1 200 OK";
        let contents = fs::read_to_string("index.html").unwrap();
        let content_length = contents.len();

        let response = format!("{status}\r\rContent-Length: {content_length}\r\n\r\n{contents}");
        stream.write_all(response.as_bytes()).unwrap();
    } else {
        let status = "HTTP/1.1 404 NOT FOUND";
        let contents = fs::read_to_string("404.html").unwrap();
        let content_length = contents.len();

        let response = format!("{status}\r\rContent-Length: {content_length}\r\n\r\n{contents}");
        stream.write_all(response.as_bytes()).unwrap();
    }
}
