use yellowstone_grpc_proto::prelude::SubscribeUpdateTransactionInfo; //it is struct that handle server connection
use sqlx::{Pool, Postgres};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use spl_token::state::Mint;
use solana_sdk::program_pack::Pack;

pub async fn process_transaction(tx_info: &SubscribeUpdateTransactionInfo, slot: u64 , db_pool:Pool<Postgres>) {
    let readable_signature = bs58::encode(&tx_info.signature).into_string();

    if let Some(transaction) = &tx_info.transaction {
        if let Some(message) = &transaction.message {
            if let Some(meta) = &tx_info.meta {
                
                // 1. Search the logs for the exact Raydium Initialize instruction!
                let mut is_new_pool = false;
                for log in &meta.log_messages {
                    if log.contains("initialize2") || log.contains("InitializeInstruction2") {
                        is_new_pool = true;
                        break;
                    }
                }

                // 2. If we found a pool launch, extract the new Token Mint!
                if is_new_pool {
                     // 2. Safely extract the Mint (usually at index 8, but our simulator only has 4 accounts)
                    // If it's a fake transaction from our simulator, we gracefully default to "Unknown"
                    let mint_pubkey_str = if message.account_keys.len() > 8 {
                        bs58::encode(&message.account_keys[8]).into_string()
                    }
                    else{
                        "Unknown_Mint_Simulator".to_string()
                    };
                    println!("\n🚨🚨🚨 NEW RAYDIUM POOL LAUNCH DETECTED! 🚨🚨🚨");
                    
                    println!("Transaction Signature: https://solscan.io/tx/{}", readable_signature);
                    println!("Token Mint: {}", mint_pubkey_str);
                    let token_name = "Unknown Token".to_string();
                    let token_symbol = "UNK".to_string();
                    let mut is_safe = false;

                     // 3. Make secondary RPC calls to fetch Rug Pull info
                    // We only do this if it's a REAL token launch (not our simulator)


                    if mint_pubkey_str != "Unknown_Mint_Simulator" {
                        let rpc_url = "https://api.mainnet-beta.solana.com"; 
                        let client = RpcClient::new(rpc_url.to_string());
                        
                        if let Ok(pubkey) = Pubkey::from_str(&mint_pubkey_str) {
                            // Fetch the Mint Account Data
                            if let Ok(account) = client.get_account(&pubkey).await {
                                if let Ok(mint_data) = Mint::unpack(&account.data) {
                                    // RUG PULL CHECK: Is the Mint Authority disabled?
                                    if mint_data.mint_authority.is_none() {
                                        is_safe = true;
                                        println!("✅ SAFETY CHECK PASSED: Mint Authority is Revoked!");
                                    } else {
                                        println!("❌ DANGER: Developer can mint infinite tokens!");
                                    }
                                }
                            }
                        }
                    } else {
                        println!("⚠️ Simulator Detected: Bypassing safety checks.");
                    }
                    println!("----------------------------------------------------\n");
                    // 4. Save this Alpha to our database!
                    let result = sqlx::query(
                        r#"
                        INSERT INTO raydium_pools (signature, slot, token_mint, token_name, token_symbol, is_safe)
                        VALUES ($1, $2, $3, $4, $5, $6)
                        ON CONFLICT (signature) DO NOTHING
                        "#
                    )
                    .bind(&readable_signature)
                    .bind(slot as i64)
                    .bind(&mint_pubkey_str)
                    .bind(&token_name)
                    .bind(&token_symbol)
                    .bind(is_safe)
                    .execute(&db_pool)
                    .await;
                    
                    if let Err(e) = result {
                        eprintln!("❌ Failed to save to Supabase: {:?}", e);
                    } else {
                        println!("💾 Saved to Supabase successfully!");
                }
            }
        }
    }
}
}
