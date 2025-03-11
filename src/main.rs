use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

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
    
    // Accept connections and process them
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Handle each client in a separate thread
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}