use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use tokio_postgres::{Error as TokioPostgresError, NoTls};

#[macro_use]
extern crate serde_derive;

// Model for user data
#[derive(Serialize, Deserialize)]
struct User {
    id: Option<i32>,
    username: String,
    email: String,
}

// constants for HTTP responses
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!";
const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\n\r\nNot Found";
const INTERNAL_SERVER_ERROR_RESPONSE: &str = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain\r\n\r\nInternal Server Error";


#[tokio::main]
async fn main() {
    // Connect to the database
    if let Err(e) = set_database().await {
        eprintln!("Error setting up the database: {}", e);
    }

    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to port 8080");    println!("Server running on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream)=>{
                handle_client(stream).await;
            }
            Err(e)=>{
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

async fn set_database() -> Result<(), TokioPostgresError> {
    let (client,connection) = tokio_postgres::connect("postgres://postgres:root@localhost/vehicle_management", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Create users table
    client.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL
        )",
        &[]
    ).await?;

    Ok(())
}

async fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let mut req = String::new();

    match stream.read(&mut buffer){
        Ok(size)=>{
            req.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());

            let (status,content) = match &*req{
                _ if req.starts_with("GET /health") => (OK_RESPONSE.to_string(), "Health check!".to_string()),
                _ if req.starts_with("GET /user/") => get_user(&req).await,
                _ if req.starts_with("GET /users/") => get_all_users().await,
                _ if req.starts_with("POST /users/")=>post_user(&req).await,
                _ if req.starts_with("DELETE /users/")=>delete_user(&req).await,
                _=>(NOT_FOUND_RESPONSE.to_string(), NOT_FOUND_RESPONSE.to_string())
            };

            stream.write_all(format!("{} {}",status,content).as_bytes()).unwrap();
        }
        Err(e)=>{
            eprintln!("Error reading from stream: {}", e);
        }
    }
}

// handler functions
async fn post_user(req: &str) -> (String, String) {
    let user = match get_user_request_body(req) {
        Ok(user) => user,
        Err(e) => return (
            "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nBad Request".to_string(),
            format!("Error parsing request body: {}", e)
        ),
    };

    let (client, connection) = match tokio_postgres::connect(
        "postgres://postgres:root@localhost/vehicle_management",
        NoTls
    ).await {
        Ok(conn) => conn,
        Err(e) => return (
            INTERNAL_SERVER_ERROR_RESPONSE.to_string(),
            format!("Error connecting to database: {}", e)
        ),
    };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Check for duplicate username or email
    match client.query_one(
        "SELECT id FROM users WHERE username = $1 OR email = $2",
        &[&user.username, &user.email]
    ).await {
        Ok(row) => {
            let existing_id: i32 = row.get(0);
            return (
                "HTTP/1.1 409 Conflict\r\nContent-Type: text/plain\r\n\r\nConflict".to_string(),
                format!("User with same username or email already exists with id: {}", existing_id)
            );
        }
        Err(e) if e.code() == Some(&tokio_postgres::error::SqlState::NO_DATA_FOUND) => {
            // No duplicate found, proceed with insert
        }
        Err(e) => return (
            INTERNAL_SERVER_ERROR_RESPONSE.to_string(),
            format!("Error checking for duplicate user: {}", e)
        ),
    }

    match client.execute(
        "INSERT INTO users (username, email) VALUES ($1, $2)",
        &[&user.username, &user.email]
    ).await {
        Ok(_) => (
            "HTTP/1.1 201 Created\r\nContent-Type: text/plain\r\n\r\nUser created".to_string(),
            "User created successfully".to_string()
        ),
        Err(e) => (
            INTERNAL_SERVER_ERROR_RESPONSE.to_string(),
            format!("Error creating user: {}", e)
        ),
    }
}

async fn get_user(req: &str) -> (String, String) {
    let user_id = match get_user_id(req).parse::<i32>() {
        Ok(id) => id,
        Err(_) => return (
            "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nBad Request".to_string(),
            "Invalid user ID".to_string()
        ),
    };

    let (client, connection) = match tokio_postgres::connect(
        "postgres://postgres:root@localhost/vehicle_management",
        NoTls
    ).await {
        Ok(conn) => conn,
        Err(e) => return (
            INTERNAL_SERVER_ERROR_RESPONSE.to_string(),
            format!("Error connecting to database: {}", e)
        ),
    };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    match client.query_one(
        "SELECT id, username, email FROM users WHERE id = $1",
        &[&user_id]
    ).await {
        Ok(row) => {
            let user = User {
                id: Some(row.get(0)),
                username: row.get(1),
                email: row.get(2),
            };
            (
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n".to_string(),
                serde_json::to_string(&user).unwrap_or_default()
            )
        }
        Err(e) => (
            NOT_FOUND_RESPONSE.to_string(),
            format!("Error fetching user: {}", e)
        ),
    }
}

async fn get_all_users() -> (String, String) {
    let (client, connection) = match tokio_postgres::connect(
        "postgres://postgres:root@localhost/vehicle_management",
        NoTls
    ).await {
        Ok(conn) => conn,
        Err(e) => return (
            INTERNAL_SERVER_ERROR_RESPONSE.to_string(),
            format!("Error connecting to database: {}", e)
        ),
    };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    match client.query(
        "SELECT id, username, email FROM users",
        &[]
    ).await {
        Ok(rows) => {
            let users: Vec<User> = rows.iter().map(|row| User {
                id: Some(row.get(0)),
                username: row.get(1),
                email: row.get(2),
            }).collect();
            (
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n".to_string(),
                serde_json::to_string(&users).unwrap_or_default()
            )
        }
        Err(e) => (
            NOT_FOUND_RESPONSE.to_string(),
            format!("Error fetching users: {}", e)
        ),
    }
}

async fn delete_user(req: &str) -> (String, String) {
    let user_id = match get_user_id(req).parse::<i32>() {
        Ok(id) => id,
        Err(_) => return (
            "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nBad Request".to_string(),
            "Invalid user ID".to_string()
        ),
    };

    let (client, connection) = match tokio_postgres::connect(
        "postgres://postgres:root@localhost/vehicle_management",
        NoTls
    ).await {
        Ok(conn) => conn,
        Err(e) => return (
            INTERNAL_SERVER_ERROR_RESPONSE.to_string(),
            format!("Error connecting to database: {}", e)
        ),
    };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    match client.execute(
        "DELETE FROM users WHERE id = $1",
        &[&user_id]
    ).await {
        Ok(rows_deleted) => {
            if rows_deleted > 0 {
                (
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nUser deleted".to_string(),
                    "User deleted successfully".to_string()
                )
            } else {
                (
                    NOT_FOUND_RESPONSE.to_string(),
                    "User not found".to_string()
                )
            }
        }
        Err(e) => (
            INTERNAL_SERVER_ERROR_RESPONSE.to_string(),
            format!("Error deleting user: {}", e)
        ),
    }
}

// get_id functions
fn get_user_id(req:&str)->&str{
    req.split("/").nth(2).unwrap_or_default().split_whitespace().next().unwrap_or_default()
}

// deserialize user data from request body
fn get_user_request_body(req:&str)->Result<User, serde_json::Error>{
    serde_json::from_str(req.split("\r\n\r\n").last().unwrap_or_default())
}