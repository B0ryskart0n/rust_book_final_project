use std::{
    io::BufReader,
    io::prelude::*,
    net::{TcpListener, TcpStream},
};

const ADDR: &str = "127.0.0.1:7878";

fn main() {
    let listener = TcpListener::bind(ADDR).expect("couldn't bind the listener");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        // A single `stream` represents an open connection between the client and the server.

        handle_connection(stream);
        // `stream` gets dropped and the connection is closed.
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);

    let request_lines: Vec<_> = buf_reader
        .lines()
        .map(|line_result| line_result.expect("request line couldn't be read as utf8"))
        .take_while(|line| !line.is_empty()) // Two newline characters signal end of HTTP request. Without this condition the iterator will not finish because the stream is open and the sender could send more stuff.
        .collect::<Vec<String>>();

    eprintln!("Request lines: {request_lines:?}");

    let response = "HTTP/1.1 200 OK\r\n\r\n";

    stream
        .write_all(response.as_bytes())
        .expect("couldn't write a response to stream");
}
