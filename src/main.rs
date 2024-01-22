use ::http_server::{RequestType, ResponseStatus, ThreadPool};
use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    path::Path,
};
fn main() {
    let listener = TcpListener::bind("192.168.1.107:8008").expect("Failed to connect");
    let pool = ThreadPool::new(4, 5);

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(x) => {eprintln!("Failed to receive: {x}"); continue;}
        };

        pool.execute(|| handle_connection(stream));
    }
    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&mut stream).lines();

    let request_line = match buf_reader.next() {
        Some(Ok(x)) => x,
        Some(Err(x)) => {
            eprintln!("Failed to read from stream, {x}");
            return;
        }
        None => "".to_string(),
    };

    let mut request_buf = request_line.split_ascii_whitespace();

    let request_type = match request_buf.next() {
        Some("GET") => RequestType::GET,
        None => {
            send_response(ResponseStatus::Fail500, stream);
            return;
        }
        Some(x) => {
            eprintln!("Not implemented {x} requests yet");
            send_response(ResponseStatus::Bad400, stream);
            return;
        } // Not Implemented this kind
    };

    // TODO: Extend this to include query and parameters
    let path = match request_buf.next() {
        Some(path) => path,
        None => {
            eprintln!("No path given");
            send_response(ResponseStatus::Bad400, stream);
            return;
        }
    };

    let http_ver = match request_buf.next() {
        Some(ver) => ver,
        None => {
            eprintln!("No HTTP ver given");
            send_response(ResponseStatus::Bad400, stream);
            return;
        }
    };

    // let header: String = buf_reader
    //     .take_while(|line| line.is_ok())
    //     .map(|x| x.unwrap())
    //     .take_while(|x| !x.is_empty())
    //     .collect();
    eprintln!("{request_type} {path} {http_ver} from: {}", match stream.peer_addr() {
        Ok(x) => x.to_string(),
        Err(x) => format!("Could not determine peer address: {x}"),
    });
    assert!(path.starts_with("/"));

    match request_type {
        RequestType::GET => send_response(ResponseStatus::Ok200(path), stream),
        _ => {
            eprintln!("Not implemented {request_type} requests yet");
            send_response(ResponseStatus::Fail500, stream)
        } // Not Implemented this kind
    }
}

#[inline]
fn send_response(response_status: ResponseStatus, mut stream: TcpStream) {
    let status_line = format!("{response_status}\r\n");
    let filename = match response_status {
        ResponseStatus::Ok200("/") => "index.html",
        ResponseStatus::Ok200(path) => path,
        ResponseStatus::Bad400 => "400.html",
        ResponseStatus::Bad404 => "404.html",
        ResponseStatus::Fail500 => "500.html",
    };

    let contents = fs::read(Path::new(&format!("./contents/{filename}"))).unwrap();

    let length = contents.len();
    let response = format!("{status_line}Content-Length: {length}\r\n\r\n");

    match stream.write_all(response.as_bytes()) {
        Ok(()) => (),
        Err(x) => {
            println!("Failed to send response {x}");
            return;
        }
    }
    match stream.write_all(&contents) {
        Ok(()) => (),
        Err(x) => {
            println!("Failed to send contents {x}");
            return;
        }
    }
    match stream.flush() {
        Ok(()) => (),
        Err(e) => println!("Failed to send all bytes: {e}"),
    }
}
