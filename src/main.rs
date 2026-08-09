use tonic::transport::ClientTlsConfig;
use futures::{SinkExt, stream::StreamExt}; //this allow .next() on our incoming data stream
use yellowstone_grpc_client::GeyserGrpcClient; //it is struct that handle server connection
use yellowstone_grpc_proto::{prelude::{
    SubscribeRequest, SubscribeRequestFilterBlocks, subscribe_update::UpdateOneof,SubscribeRequestFilterTransactions,
}, tonic::Request};
use std::{collections::HashMap, hash::Hash};// Geyser protocol requires us to name our subscriptions using key-value pairs.

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main] //async runtime of rust, this turns main into async main
async fn main() -> Result<(),Box<dyn std::error::Error>>{  //we can use "?" to throw back the error
    tracing_subscriber::fmt::init(); //initialize tracing for logs
    dotenv().ok(); //load the env
    let db_url=env::var("DATABASE_URL").expect("Database Url must be set in env"); //Connecting to supabase url
    println!("Connecting to Supabase Database...");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    println!("Successfully connected to Supabase!");


    let endpoint = "https://laserstream-devnet-ewr.helius-rpc.com".to_string();
    // For gRPC, we MUST pass the API key in the x-token header, not the URL!
    let x_token = Some("ab77d0f0-033d-4753-87a3-ffedebec057a".to_string());  //x_token has my api key
    println!("Connecting to Helius Geyser gRPC...");
    //Geyser client
    let mut client = GeyserGrpcClient::build_from_shared(endpoint)?
        .x_token(x_token)?
        .tls_config(ClientTlsConfig::new().with_webpki_roots())? //We use with_webpki_roots() to let the client know how to verify standard web SSL certificates
        .connect()
        .await?;
    println!("Successfully connected to Helius!");
    //create Transaction Table if it doesn't exist
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

    // //A filter for "Blocks" (Day 1)
    // let mut blocks = HashMap::new();
    // blocks.insert("abc_subs".to_string(), SubscribeRequestFilterBlocks{
    //     account_include:vec![],
    //     include_transactions:Some(false),
    //     include_accounts:Some(false),
    //     include_entries:Some(false),
    //     cuckoo_account_include: None,
    // });

    //A filter for Transactions (Day 2 onwards)
    let mut transactions = HashMap::new();
    transactions.insert("whirlpool_txs".to_string(), SubscribeRequestFilterTransactions{
        vote:Some(false), //Validator vote transaction not needed
        failed:Some(false), //we need only the successful transactions
        signature:None,
        account_include: vec!["TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string()], //TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA | CmcT8bch4VwsxHqWEvKGzGtCZdKCeYSZVxhjw5quGjzn
        account_exclude:vec![],
        account_required:vec![],
        token_accounts:None,
    },);

    //Pack the filter into the main Geyser SubscribeRequest object
    let request = SubscribeRequest{
        blocks:HashMap::new(), //emptied this out because we only want transactions now
        accounts: HashMap::new(),
        accounts_data_slice: vec![],
        transactions,
        transactions_status: HashMap::new(),
        slots: HashMap::new(),
        entry: HashMap::new(),
        blocks_meta: HashMap::new(),

        // 1 = Confirmed (safe from minor forks). 2 = Finalized (100% safe, but slower). 
        // 0 = Processed (fastest, but blocks might get rolled back).
        commitment: Some(1), 
        from_slot:None,
        ping: None,
    };

     println!("Sending subscription request to Helius...");

     //Sending the subscription request and get a stream back
     let (mut subscribe_tx, mut stream) = client.subscribe().await?;
     subscribe_tx.send(request).await?;

    //Listening to the incoming stream of data forever
        println!("Listening for new blocks... (Press Ctrl+C to stop)");
        while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => {
                // Look inside the message to see what Helius sent us
                if let Some(update) = msg.update_oneof {
                    match update {
                        UpdateOneof::Transaction(tx) => {
                            if let Some(tx_info) = tx.transaction{
                                let raw_signature = tx_info.signature;
                                let readable_signature = bs58::encode(raw_signature).into_string();
                                println!(
                                    "Caught a Transaction in SPL Token! \nSlot: {}\nSignature: https://solscan.io/tx/{}?cluster=devnet\n", 
                                    tx.slot, 
                                    readable_signature);       
                                if let Some(transaction) = tx_info.transaction{
                                    if let Some(message) = transaction.message{
                                            println!("--- TRANSACTION DETAILS ---");
                                            println!("Number of Instructions: {}", message.instructions.len());
                                            println!("Accounts involved (first 3):");
                                            if let Some(meta) = &tx_info.meta {
                                                // Prepare arrays to hold our data for Postgres
                                                let mut accounts_involved: Vec<String> = Vec::new();
                                                let mut balance_changes: Vec<f64> = Vec::new();

                                                for (i, raw_pubkey) in message.account_keys.iter().enumerate() {
                                                    let readable_pubkey = bs58::encode(raw_pubkey).into_string();
                                                    let change_lamports = (meta.post_balances[i] as i64) - (meta.pre_balances[i] as i64);
                                                    let change_sol = (change_lamports as f64) / 1_000_000_000.0;

                                                    // We'll just save the wallets that actually had a balance change to save space!
                                                    if change_sol != 0.0 {
                                                        accounts_involved.push(readable_pubkey);
                                                        balance_changes.push(change_sol);
                                                    }
                                                }

                                                // Only save to the database if money actually moved
                                                if !accounts_involved.is_empty() {
                                                    let instruction_count = message.instructions.len() as i32;
                                                    let slot_num = tx.slot as i64;
                                                    
                                                    // Insert into Supabase in the background
                                                    let pool = db_pool.clone();
                                                    let sig = readable_signature.clone();
                                                    
                                                    tokio::spawn(async move {
                                                        let result = sqlx::query(
                                                            r#"
                                                            INSERT INTO transactions (signature, slot, instruction_count, accounts_involved, balance_changes)
                                                            VALUES ($1, $2, $3, $4, $5)
                                                            ON CONFLICT (signature) DO NOTHING
                                                            "#
                                                        )
                                                        .bind(&sig)
                                                        .bind(slot_num)
                                                        .bind(instruction_count)
                                                        .bind(&accounts_involved)
                                                        .bind(&balance_changes)
                                                        .execute(&pool)
                                                        .await;

                                                        match result {
                                                            Ok(_) => println!("💾 Saved transaction {} to Supabase!", sig),
                                                            Err(e) => eprintln!("❌ Failed to save to DB: {:?}", e),
                                                        }
                                                    });
                                                }
                                            }

                                    }
                                }
                                                     }
                        },
                        UpdateOneof::Ping(_) => {
                            // Helius sends a ping every few seconds to keep the connection open
                            println!("Received ping from server");
                        },
                        _ => {
                            println!("Received other update type");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Stream error: {:?}", e);
                break; // Exit the loop if our internet cuts out
            }
        }
    }

    Ok(())
}




