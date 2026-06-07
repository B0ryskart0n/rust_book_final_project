use rust_book_final_project::ThreadPool;

use std::{
    fs,
    io::{self, BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

const ADDR: &str = "127.0.0.1:7878";

fn main() -> io::Result<()> {
    let listener = TcpListener::bind(ADDR)?;
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        eprintln!("Incomming TCP connection.");
        let stream = stream?;
        // A single `stream` represents an open connection between the client and the server.
        eprintln!("Established TCP connection: {stream:?}.");

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    eprintln!("Shutting down service.");
    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);

    let request_lines: Vec<_> = buf_reader
        .lines()
        .map(|line_result| line_result.expect("request line should be readable and proper utf8"))
        .take_while(|line| !line.is_empty()) // Two newline characters signal end of HTTP request. Without this condition the iterator will not finish because the stream is open and the sender could send more stuff.
        .collect::<Vec<String>>();

    let request_status_line = request_lines.get(0);
    eprintln!("  Request status line: {request_status_line:?}.",);

    let (status, filename) = match request_status_line.map(|string| &string[..]) {
        Some("GET / HTTP/1.1") => ("HTTP/1.1 200 OK", "hello.html"),
        Some("GET /sleep HTTP/1.1") => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        Some(_) => ("HTTP/1.1 404 NOT FOUND", "404.html"),
        // It seems like the browser tries to always maintain a connection, but doesn't send anything if it already has the content, so closing the browser gives an empty request.
        None => {
            eprintln!("  Empty stream closed: returning.");
            return;
        }
    };

    let contents = fs::read_to_string(filename).expect("html file should be readable");
    let length = contents.len();

    // CRLFCRLF should separate the headers and the contents.
    let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream
        .write_all(response.as_bytes())
        .expect("open stream should be writable");
    eprintln!("  Sent resposne.");

    // `stream` gets dropped and the connection is closed.
    eprintln!("Closing the connection: {stream:?}.");
}
