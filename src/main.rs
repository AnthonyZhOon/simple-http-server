use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
    
    
};
use http_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("192.168.1.107:8008").expect("Failed to connect");
    let pool = ThreadPool::new(4, 5);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| handle_connection(stream));
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = match buf_reader.lines().next() {
        Some(x) => x.unwrap_or_else(|err| {println!("{:#?}", err);
        "".to_string()
    }),
        None => "".to_string()
    };
    // todo!("Make a helper function for checking the request line, return the req, path, args, HTTP ver");
    //  I know regex is the best choice for matching request_line but I want to implement from scratch
    // Log the client IP?
    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /cat.jpg HTTP/1.1" => ("HTTP/1.1 200 OK", "cat.jpg"),
        "GET /CV.pdf HTTP/1.1" => ("HTTP/1.1 200 OK", "CV.pdf"),
        "GET /icon.png HTTP/1.1" => ("HTTP/1.1 200 OK", "icon.png"),
        // "GET /sleep HTTP/1.1" => {
        //     thread::sleep(Duration::from_secs(5));
        //     ("HTTP/1.1 200 OK", "hello.html")
        // }
        "" => ("HTTP/1.1 400 BAD REQUEST", "400.html"),
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),

    };

    let contents = fs::read(filename).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n");

    stream.write_all(response.as_bytes()).unwrap();
    stream.write_all(&contents).unwrap();
    stream.flush().unwrap();
}
