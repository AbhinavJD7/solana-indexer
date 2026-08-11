use std::env;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

// We make this function `pub` so main.rs can see it!
// It returns a connected Postgres Pool if successful.
pub async fn setup_database() -> Result<Pool<Postgres>, Box<dyn std::error::Error>> {
    
    // 1. Load the database URL from the environment
    let db_url = env::var("DATABASE_URL").expect("Database Url must be set in env");
    
    println!("Connecting to Supabase Database...");
    
    // 2. Establish the connection pool
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
        
    println!("Successfully connected to Supabase!");

    // 3. Ensure the schema exists before we start ingesting data
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS transactions (
            signature VARCHAR(88) PRIMARY KEY,
            slot BIGINT NOT NULL,
            instruction_count INT NOT NULL,
            accounts_involved TEXT[] NOT NULL,
            balance_changes DOUBLE PRECISION[] NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        );
        "#
    )
    .execute(&db_pool)
    .await?;
    
    println!("Database schema is ready!");

    // Return the active connection pool back to main.rs
    Ok(db_pool)
}
