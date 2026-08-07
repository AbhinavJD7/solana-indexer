use tonic::transport::ClientTlsConfig;
use futures::{SinkExt, stream::StreamExt}; //this allow .next() on our incoming data stream
use yellowstone_grpc_client::GeyserGrpcClient; //it is struct that handle server connection
use yellowstone_grpc_proto::{prelude::{
    SubscribeRequest, SubscribeRequestFilterBlocks, subscribe_update::UpdateOneof,
}, tonic::Request};
use std::collections::HashMap;// Geyser protocol requires us to name our subscriptions using key-value pairs.

#[tokio::main] //async runtime of rust, this turns main into async main
async fn main() -> Result<(),Box<dyn std::error::Error>>{  //we can use "?" to throw back the error
    tracing_subscriber::fmt::init(); //initialize tracing for logs

    let endpoint = "https://laserstream-devnet-ewr.helius-rpc.com".to_string();
    // For gRPC, we MUST pass the API key in the x-token header, not the URL!
    let x_token = Some("ab77d0f0-033d-4753-87a3-ffedebec057a".to_string()); 
    println!("Connecting to Helius Geyser gRPC...");
    //Geyser client
    let mut client = GeyserGrpcClient::build_from_shared(endpoint)?
        .x_token(x_token)?
        .tls_config(ClientTlsConfig::new().with_webpki_roots())? //We use with_webpki_roots() to let the client know how to verify standard web SSL certificates
        .connect()
        .await?;
    println!("Successfully connected to Helius!");

    //A filter for "Blocks"
    let mut blocks = HashMap::new();
    blocks.insert("abc_subs".to_string(), SubscribeRequestFilterBlocks{
        account_include:vec![],
        include_transactions:Some(false),
        include_accounts:Some(false),
        include_entries:Some(false),
        cuckoo_account_include: None,
    });

    //Pack the filter into the main Geyser SubscribeRequest object
    let request = SubscribeRequest{
        blocks,
        accounts: HashMap::new(),
        accounts_data_slice: vec![],
        transactions: HashMap::new(),
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
                        UpdateOneof::Block(block) => {
                            // We caught a block!
                            println!("Received new block at slot: {}", block.slot);
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




