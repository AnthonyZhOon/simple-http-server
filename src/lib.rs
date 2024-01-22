use std::{
    error::Error,
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}


pub enum RequestType {
    GET,
    POST,
    HEAD,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl std::fmt::Display for RequestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            Self::GET => "GET",
            Self::POST => "POST",
            Self::HEAD => "HEAD",
            Self::PUT => "PUT",
            Self::DELETE => "DELETE",
            Self::CONNECT => "CONNECT",
            Self::OPTIONS => "OPTIONS",
            Self::TRACE => "TRACE",
            Self::PATCH => "PATCH",
        };
        write!(f, "{x}")
    }
}
pub enum ResponseStatus<'a> {
    Ok200(&'a str),
    Bad400,
    Bad404,
    Fail500
}
impl<'a> std::fmt::Display for ResponseStatus<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            Self::Ok200(_) => "HTTP/1.1 200 OK",
            Self::Bad404 => "HTTP/1.1 404 NOT FOUND",
            Self::Bad400 => "HTTP/1.1 400 BAD REQUEST",
            Self::Fail500 => "HTTP/1.1 500 INTERNAL SERVER ERROR"
        };
        write!(f, "{x}")
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize, tries: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        'outer: for id in 0..size {
            for _ in 0..tries {
                if let Ok(worker) = Worker::new(id, Arc::clone(&receiver)) {
                    workers.push(worker);
                    continue 'outer;
                }
            }
            panic!("Could not create thread after retrying");
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap(); // Does not fail as the corresponding sender always exists until server shutdown, we know receiver will outlive sender
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().expect(format!("Couldn't join thread for worker {}", worker.id).as_ref());
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

#[derive(Debug)]
pub struct ThreadSpawnError {
    id: usize,
}

impl Error for ThreadSpawnError {}

impl std::fmt::Display for ThreadSpawnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OS failed to spawn thread for worker {}", self.id)
    }
}
impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
    ) -> Result<Worker, ThreadSpawnError> {
        let builder = thread::Builder::new();
        match builder.spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    eprint!("Worker {id} got a job; executing. ");

                    job();
                }
                Err(_) => {
                    eprintln!("Worker {id} disconnected; shutting donw.");
                    break;
                }
            }
        }) {
            Ok(thread) => Ok(Worker {
                id,
                thread: Some(thread),
            }),
            Err(_) => Err(ThreadSpawnError { id }),
        }
    }
}
