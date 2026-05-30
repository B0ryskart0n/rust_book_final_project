use std::net::TcpListener;

const ADDR: &str = "127.0.0.1:7878";

fn main() {
    let listener = TcpListener::bind(ADDR).expect("Couldn't bind the listener");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        // a single `stream` represents an open connection between the client and the server.

        println!("Connection established!");
        // `stream` gets dropped and the connection is closed.
    }
}
