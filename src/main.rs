use ::http_server::{RequestType, ResponseStatus, ThreadPool};
use std::{
    env::args,
    fs,
    io::{prelude::*, BufReader, ErrorKind::NotFound},
    net::{TcpListener, TcpStream},
    path::Path,
};

fn help() {
    println!("-h | --help: Displays help\n--hostname: Input a hostname/ip address, default is 127.0.0.1 (localhost)\n--port: Specify the port to connect to, default is 8080")
}
fn main() {
    let mut hostname = "127.0.0.1".to_string();
    let mut port = "8080".to_string();

    let mut args = args().skip(1);
    while let Some(arg) = args.next() {
        match &arg[..] {
            "-h" | "--help" => help(),
            "--version" => {
                println!("{} {}", "simple-http-server", "VERSION");
            }
            "-q" | "--quiet" => {
                println!("Quiet mode is not supported yet.");
            }
            "-v" | "--verbose" => {
                println!("Verbose mode is not supported yet.");
            }
            "--hostname" => {
                if let Some(ip) = args.next() {
                    hostname = ip;
                } else {
                    panic!("No value specified for parameter --hostname.");
                }
            }
            "--port" => {
                if let Some(p) = args.next() {
                    port = p;
                } else {
                    panic!("No value specified for parameter --port.");
                }
            }
            _ => {
                if arg.starts_with('-') {
                    println!("Unkown argument {}", arg);
                } else {
                    println!("Unkown positional argument {}", arg);
                }
            }
        }
    }

    let listener = TcpListener::bind(format!("{hostname}:{port}")).expect("Failed to connect");
    eprintln!("Listening to {}", listener.local_addr().unwrap());
    let pool = ThreadPool::new(4, 5);

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(x) => {
                eprintln!("Failed to receive: {x}");
                continue;
            }
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

    eprintln!(
        "{request_type} {path} {http_ver} from: {}",
        match stream.peer_addr() {
            Ok(x) => x.to_string(),
            Err(x) => format!("Could not determine peer address: {x}"),
        }
    );

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
        ResponseStatus::Ok200(path) => path.get(1..).unwrap(),
        ResponseStatus::Bad400 => "400.html",
        ResponseStatus::Bad404 => "404.html",
        ResponseStatus::Fail500 => "500.html",
    };

    if filename.contains("..") { 
        eprintln!("Attemped bad request trying to navigate up filesystem");
        send_response(ResponseStatus::Bad404, stream);
        return;
    }
    match fs::read_dir("contents/") {
        Ok(dir) => {
            if !dir
                .filter_map(|x| if x.is_ok() { Some(x.unwrap()) } else { None })
                .any(|entry| entry.path().ends_with(filename))
            {
                eprintln!("Attempted to access illegal file {filename}");
                send_response(ResponseStatus::Bad404, stream);
                return
            }
        }
        Err(e) => {
            eprintln!("contents/ not found: {e}");
            send_response(ResponseStatus::Bad404, stream);
            return;
        }
    }

    let contents = match fs::read(Path::new(&format!("./contents/{filename}"))) {
        Ok(x) => x,
        Err(e) => match e.kind() {
            NotFound => {
                send_response(ResponseStatus::Bad404, stream);
                return;
            }
            _ => {
                send_response(ResponseStatus::Bad400, stream);
                return;
            }
        },
    };

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
