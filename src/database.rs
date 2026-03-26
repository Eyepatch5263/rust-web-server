use tokio_postgres::{ Error as TokioPostgresError, NoTls };

pub async fn set_database() -> Result<(), TokioPostgresError> {
    let (client, connection) = tokio_postgres::connect(
        "postgres://postgres:root@localhost/vehicle_management",
        NoTls
    ).await?;

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