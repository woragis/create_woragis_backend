use std::{env, sync::Arc};

use log::debug;
use tokio::sync::Mutex;
use tokio_postgres::{Client, Error, NoTls};


/// Table names for the database.
pub static USERS_TABLE: &str = "users";
pub static TODOS_TABLE: &str = "todos";

/// Establishes a connection to the PostgreSQL database.
///
/// # Returns
/// A `Result` containing the `Client` for executing queries or an error if the connection fails.
///
/// # Errors
/// Returns an `Error` if the database connection cannot be established.
pub async fn connect() -> Result<Client, Error> {
    // Fetch the database URL from environment variables.
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Log the database URL for debugging purposes (consider masking sensitive data).
    debug!("Database url found: {}", database_url);

    // Attempt to connect to the PostgreSQL database.
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;

    // Spawn a separate task to manage the connection and handle potential errors.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            log::error!("Database connection error: {}", e);
        }
    });

    Ok(client)
}

/// Creates necessary database tables if they do not exist.
///
/// # Parameters
/// - `client`: A shared and synchronized PostgreSQL client.
///
/// # Returns
/// A `Result` indicating success or failure in creating the tables.
///
/// # Errors
/// Returns an `Error` if any of the table creation queries fail.
pub async fn create_tables(client: &Arc<Mutex<Client>>) -> Result<(), Error> {
    // Log the table creation process.
    debug!("Creating tables: '{}'", USERS_TABLE);

    // Ensure the pgcrypto extension is available for UUID generation.
    let extension = "CREATE EXTENSION IF NOT EXISTS pgcrypto;";

    // SQL statement for creating the users table.
    let create_users_table = format!(
        "
    CREATE TABLE IF NOT EXISTS {} (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        name VARCHAR(100) NOT NULL,
        email_hash CHAR(128) NOT NULL UNIQUE,
        email_encrypt TEXT NOT NULL,
        nonce VARCHAR(24) NOT NULL,
        password TEXT NOT NULL,
        role VARCHAR(5) NOT NULL CHECK (role IN ('admin', 'user')) DEFAULT 'user',
    );
    ",
        USERS_TABLE
    );

    // Lock the database client to execute queries sequentially.
    let client = client.lock().await;

    // Execute the extension and table creation queries.
    client
        .batch_execute(&extension)
        .await
        .expect("Could not pgcrypto create extension");
    client
        .batch_execute(&create_users_table)
        .await
        .expect("Could not create users table");

    Ok(())
}
