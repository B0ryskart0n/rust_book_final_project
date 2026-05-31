use rust_book_final_project::ThreadPool;

use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

const ADDR: &str = "127.0.0.1:7878";

fn main() {
    let listener = TcpListener::bind(ADDR).expect("couldn't bind the listener");
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        eprintln!("Incomming TCP connection.");
        let stream = stream.unwrap();
        // A single `stream` represents an open connection between the client and the server.
        eprintln!("Established TCP connection: {stream:?}.");

        pool.execute(|| {
            handle_connection(stream);
        });
        // `stream` gets dropped and the connection is closed.
        eprintln!("Closing the connection.")
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);

    let request_lines: Vec<_> = buf_reader
        .lines()
        .map(|line_result| line_result.expect("request line couldn't be read as utf8"))
        .take_while(|line| !line.is_empty()) // Two newline characters signal end of HTTP request. Without this condition the iterator will not finish because the stream is open and the sender could send more stuff.
        .collect::<Vec<String>>();
    eprintln!("Request lines: {request_lines:?}.");

    // It seems like the browser tries to always maintain a connection, but doesn't send anything if it already has the content, so closing the browser gives an empty request.
    let (status, filename) = match request_lines.get(0).map(|string| &string[..]) {
        Some("GET / HTTP/1.1") => ("HTTP/1.1 200 OK", "hello.html"),
        Some("GET /sleep HTTP/1.1") => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        Some(_) => ("HTTP/1.1 404 NOT FOUND", "404.html"),
        None => {
            eprintln!("Empty stream closed: returning.");
            return;
        }
    };

    let contents = fs::read_to_string(filename).expect("couldn't read html file");
    let length = contents.len();

    // CRLFCRLF should separate the headers and the contents.
    let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream
        .write_all(response.as_bytes())
        .expect("couldn't write a response to stream");
    eprintln!("Sent resposne.");
}
