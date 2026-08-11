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
pub mod database;
pub mod grpc;
pub mod processor;

#[tokio::main] //async runtime of rust, this turns main into async main
async fn main() -> Result<(),Box<dyn std::error::Error>>{  //we can use "?" to throw back the error
    tracing_subscriber::fmt::init(); //initialize tracing for logs
    dotenv().ok(); //load the env
    let db_pool = database::setup_database().await?;
    let endpoint = "https://laserstream-devnet-ewr.helius-rpc.com"; 
    let api_key = "ab77d0f0-033d-4753-87a3-ffedebec057a";
    
    let mut client = grpc::setup_geyser_client(endpoint, api_key).await?;
    let request = grpc::build_raydium_filter();

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
                            if let Some(tx_info) = tx.transaction {
                                processor::process_transaction(&tx_info, tx.slot);
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




