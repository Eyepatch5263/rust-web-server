use std::net::{ TcpListener, TcpStream };
use std::io::{ Read, Write };
mod handler;
mod utility;
mod constant;
mod database;
use handler::{ post_user, get_user, get_all_users, delete_user };

use crate::constant::{NOT_FOUND_RESPONSE, OK_RESPONSE};
use crate::database::set_database;

#[macro_use]
extern crate serde_derive;
// Model for user data
#[derive(Serialize, Deserialize)]
struct User {
    id: Option<i32>,
    username: String,
    email: String,
}

#[tokio::main]
async fn main() {
    // Connect to the database
    if let Err(e) = set_database().await {
        eprintln!("Error setting up the database: {}", e);
    }

    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to port 8080");
    println!("Server running on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream).await;
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

async fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let mut req = String::new();

    match stream.read(&mut buffer) {
        Ok(size) => {
            req.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());

            let (status, content) = match &*req {
                _ if req.starts_with("GET /health") =>
                    (OK_RESPONSE.to_string(), "Health check!".to_string()),
                _ if req.starts_with("GET /user/") => get_user(&req).await,
                _ if req.starts_with("GET /users/") => get_all_users().await,
                _ if req.starts_with("POST /users/") => post_user(&req).await,
                _ if req.starts_with("DELETE /users/") => delete_user(&req).await,
                _ => (NOT_FOUND_RESPONSE.to_string(), NOT_FOUND_RESPONSE.to_string()),
            };

            stream.write_all(format!("{} {}", status, content).as_bytes()).unwrap();
        }
        Err(e) => {
            eprintln!("Error reading from stream: {}", e);
        }
    }
}
