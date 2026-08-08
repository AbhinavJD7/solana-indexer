use tonic::transport::ClientTlsConfig;
use futures::{SinkExt, stream::StreamExt}; //this allow .next() on our incoming data stream
use yellowstone_grpc_client::GeyserGrpcClient; //it is struct that handle server connection
use yellowstone_grpc_proto::{prelude::{
    SubscribeRequest, SubscribeRequestFilterBlocks, subscribe_update::UpdateOneof,SubscribeRequestFilterTransactions,
}, tonic::Request};
use std::{collections::HashMap, hash::Hash};// Geyser protocol requires us to name our subscriptions using key-value pairs.

#[tokio::main] //async runtime of rust, this turns main into async main
async fn main() -> Result<(),Box<dyn std::error::Error>>{  //we can use "?" to throw back the error
    tracing_subscriber::fmt::init(); //initialize tracing for logs

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
                                            if let Some(meta) = &tx_info.meta{
                                                println!("\n SOL Balance Changes:");
                                                //We loop through the first 3 accounts again
                                                for i in 0..std::cmp::min(3, message.account_keys.len()) {
                                                    let pre_lamports = meta.pre_balances[i];
                                                    let post_lamports = meta.post_balances[i];
                                                    
                                                    // Calculate the difference (cast to i64 to handle negative numbers)
                                                    let change_lamports = (post_lamports as i64) - (pre_lamports as i64);
                                                    
                                                    // Convert Lamports to SOL
                                                    let change_sol = change_lamports as f64 / 1_000_000_000.0;
                                                    
                                                    let readable_pubkey = bs58::encode(&message.account_keys[i]).into_string();
                                                    
                                                    println!("  Wallet: {}", readable_pubkey);
                                                    println!("  Change: {} SOL", change_sol);
                                                    println!("  ---");
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




