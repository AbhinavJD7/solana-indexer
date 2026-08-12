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

    // Create the specialized raydium_pools table!
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS raydium_pools (
            signature VARCHAR(88) PRIMARY KEY,
            slot BIGINT NOT NULL,
            token_mint VARCHAR(88) NOT NULL,
            token_name VARCHAR(100),
            token_symbol VARCHAR(20),
            is_safe BOOLEAN NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );
        "#
    )
    .execute(&db_pool)
    .await?;
    println!("Database schema is ready!");
    Ok(db_pool)
}
