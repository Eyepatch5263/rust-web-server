use tokio_postgres::{ NoTls };

use crate::constant::{INTERNAL_SERVER_ERROR_RESPONSE,NOT_FOUND_RESPONSE};
use crate::User;
use crate::utility::{ get_user_id, get_user_request_body };

// handler functions
pub async fn post_user(req: &str) -> (String, String) {
    let user = match get_user_request_body(req) {
        Ok(user) => user,
        Err(e) => {
            return (
                "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nBad Request".to_string(),
                format!("Error parsing request body: {}", e),
            );
        }
    };

    let (client, connection) = match
        tokio_postgres::connect(
            "postgres://postgres:root@localhost/vehicle_management",
            NoTls
        ).await
    {
        Ok(conn) => conn,
        Err(e) => {
            return (
                INTERNAL_SERVER_ERROR_RESPONSE.to_string(),
                format!("Error connecting to database: {}", e),
            );
        }
    };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Check for duplicate username or email
    let rows = match
        client.query(
            "SELECT id FROM users WHERE username = $1 OR email = $2",
            &[&user.username, &user.email]
        ).await
    {
        Ok(rows) => rows,
        Err(e) => {
            return (
                INTERNAL_SERVER_ERROR_RESPONSE.to_string(),
                format!("Error checking for duplicate user: {}", e),
            );
        }
    };

    // Now check if any rows came back
    if let Some(row) = rows.first() {
        let existing_id: i32 = row.get(0);
        return (
            "HTTP/1.1 409 Conflict\r\nContent-Type: text/plain\r\n\r\nConflict".to_string(),
            format!("User with same username or email already exists with id: {}", existing_id),
        );
    }

    match
        client.execute(
            "INSERT INTO users (username, email) VALUES ($1, $2)",
            &[&user.username, &user.email]
        ).await
    {
        Ok(_) =>
            (
                "HTTP/1.1 201 Created\r\nContent-Type: text/plain\r\n\r\nUser created".to_string(),
                "User created successfully".to_string(),
            ),
        Err(e) =>
            (INTERNAL_SERVER_ERROR_RESPONSE.to_string(), format!("Error creating user: {}", e)),
    }
}

pub async fn get_user(req: &str) -> (String, String) {
    let user_id = match get_user_id(req).parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            return (
                "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nBad Request".to_string(),
                "Invalid user ID".to_string(),
            );
        }
    };

    let (client, connection) = match
        tokio_postgres::connect(
            "postgres://postgres:root@localhost/vehicle_management",
            NoTls
        ).await
    {
        Ok(conn) => conn,
        Err(e) => {
            return (
                INTERNAL_SERVER_ERROR_RESPONSE.to_string(),
                format!("Error connecting to database: {}", e),
            );
        }
    };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    match
        client.query_one("SELECT id, username, email FROM users WHERE id = $1", &[&user_id]).await
    {
        Ok(row) => {
            let user = User {
                id: Some(row.get(0)),
                username: row.get(1),
                email: row.get(2),
            };
            (
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n".to_string(),
                serde_json::to_string(&user).unwrap_or_default(),
            )
        }
        Err(e) => (NOT_FOUND_RESPONSE.to_string(), format!("Error fetching user: {}", e)),
    }
}

pub async fn get_all_users() -> (String, String) {
    let (client, connection) = match
        tokio_postgres::connect(
            "postgres://postgres:root@localhost/vehicle_management",
            NoTls
        ).await
    {
        Ok(conn) => conn,
        Err(e) => {
            return (
                INTERNAL_SERVER_ERROR_RESPONSE.to_string(),
                format!("Error connecting to database: {}", e),
            );
        }
    };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    match client.query("SELECT id, username, email FROM users", &[]).await {
        Ok(rows) => {
            let users: Vec<User> = rows
                .iter()
                .map(|row| User {
                    id: Some(row.get(0)),
                    username: row.get(1),
                    email: row.get(2),
                })
                .collect();
            (
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n".to_string(),
                serde_json::to_string(&users).unwrap_or_default(),
            )
        }
        Err(e) => (NOT_FOUND_RESPONSE.to_string(), format!("Error fetching users: {}", e)),
    }
}

pub async fn delete_user(req: &str) -> (String, String) {
    let user_id = match get_user_id(req).parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            return (
                "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nBad Request".to_string(),
                "Invalid user ID".to_string(),
            );
        }
    };

    let (client, connection) = match
        tokio_postgres::connect(
            "postgres://postgres:root@localhost/vehicle_management",
            NoTls
        ).await
    {
        Ok(conn) => conn,
        Err(e) => {
            return (
                INTERNAL_SERVER_ERROR_RESPONSE.to_string(),
                format!("Error connecting to database: {}", e),
            );
        }
    };

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    match client.execute("DELETE FROM users WHERE id = $1", &[&user_id]).await {
        Ok(rows_deleted) => {
            if rows_deleted > 0 {
                (
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nUser deleted".to_string(),
                    "User deleted successfully".to_string(),
                )
            } else {
                (NOT_FOUND_RESPONSE.to_string(), "User not found".to_string())
            }
        }
        Err(e) =>
            (INTERNAL_SERVER_ERROR_RESPONSE.to_string(), format!("Error deleting user: {}", e)),
    }
}
