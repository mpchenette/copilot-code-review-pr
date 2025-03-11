use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

// Define a type that will be sent through the channel
enum Message {
    NewJob(Job),
    Terminate,
}

// Define the Job type as a trait object that can be called
type Job = Box<dyn FnOnce() + Send + 'static>;

// Thread Pool implementation
struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers...");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers...");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

// Worker implementation
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    // Buffer to store the request
    let mut buffer = [0; 1024];
    
    // Read the request
    match stream.read(&mut buffer) {
        Ok(_) => {
            // Convert the request to a string for easier inspection
            let request = String::from_utf8_lossy(&buffer[..]);
            println!("Request received:\n{}", request);
            
            // Prepare a simple HTTP response
            let response = "HTTP/1.1 200 OK\r\n\
                           Content-Type: text/html; charset=UTF-8\r\n\
                           \r\n\
                           <html><body>\
                           <h1>Hello from Rust simple web server!</h1>\
                           <p>This is a basic web server using only the standard library.</p>\
                           <p>Now with an efficient thread pool!</p>\
                           </body></html>\r\n";
            
            // Send the response
            match stream.write(response.as_bytes()) {
                Ok(_) => println!("Response sent successfully"),
                Err(e) => println!("Failed to send response: {}", e),
            }
        }
        Err(e) => println!("Error reading from connection: {}", e),
    }
}

fn main() {
    // Create a TCP listener on port 8000
    let listener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind to address");
    println!("Server listening on http://127.0.0.1:8000/");
    
    // Create a new thread pool with 4 threads
    let pool = ThreadPool::new(4);
    println!("Thread pool initialized with 4 workers");
    
    // Accept connections and process them
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Instead of spawning a new thread for each request,
                // use our thread pool to handle the connection
                pool.execute(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
    
    // The pool will go out of scope here and the Drop trait will clean up the threads
}