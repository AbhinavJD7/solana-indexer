use std::collections::HashMap; //// Geyser protocol requires us to name our subscriptions using key-value pairs.
use yellowstone_grpc_client::GeyserGrpcClient;
use tonic::transport::ClientTlsConfig;
use yellowstone_grpc_proto::prelude::{
    SubscribeRequest, SubscribeRequestFilterTransactions,
};

// This function sets up the connection and returns the client
pub async fn setup_geyser_client(endpoint: &str, api_key: &str) -> Result<GeyserGrpcClient, Box<dyn std::error::Error>> {
    println!("Connecting to Helius Geyser gRPC...");
    let client = GeyserGrpcClient::build_from_shared(endpoint.to_string())?
        .x_token(Some(api_key.to_string()))?
        .tls_config(ClientTlsConfig::new().with_webpki_roots())?
        .connect()
        .await?;
    
    println!("Successfully connected to Helius!");
    Ok(client)
}

// This function builds our highly specific Raydium Sniper filter
pub fn build_raydium_filter() -> SubscribeRequest {
    let mut transactions = HashMap::new();
    
    transactions.insert("raydium_pool_launches".to_string(), SubscribeRequestFilterTransactions {
        vote: Some(false), 
        failed: Some(false), 
        signature: None,
        account_include: vec![
            // 🚨 ALPHA SECRET: The Raydium "Create Pool Fee" account!
            "7YttLkHDoNj9wyDur5pM1ejNaAvT9X4eqaYcHQqtj2G5".to_string()
        ],
        account_exclude: vec![],
        account_required: vec![],
        token_accounts: None,
    });

    SubscribeRequest {
        blocks: HashMap::new(),
        accounts: HashMap::new(),
        accounts_data_slice: vec![],
        transactions,
        transactions_status: HashMap::new(),
        slots: HashMap::new(),
        entry: HashMap::new(),
        blocks_meta: HashMap::new(),
        commitment: Some(1), 
        from_slot: None,
        ping: None,
    }
}
