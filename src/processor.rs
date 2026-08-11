use yellowstone_grpc_proto::prelude::SubscribeUpdateTransactionInfo;

pub fn process_transaction(tx_info: &SubscribeUpdateTransactionInfo, slot: u64) {
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
                    println!("\n🚨🚨🚨 NEW RAYDIUM POOL LAUNCH DETECTED! 🚨🚨🚨");
                    
                    println!("Transaction Signature: https://solscan.io/tx/{}", readable_signature);
                    println!("Slot: {}", slot);
                    
                    println!("Accounts involved (Token Mints will be in here):");
                    // We print every account touched. The new Token Mint is usually at index 8 or 9!
                    for (i, raw_pubkey) in message.account_keys.iter().enumerate() {
                        let readable_pubkey = bs58::encode(raw_pubkey).into_string();
                        println!("  [{}] {}", i, readable_pubkey);
                    }
                    
                    println!("----------------------------------------------------\n");
                    
                    // WE TEMPORARILY DISABLED SUPABASE INSERTS TO PROTECT YOUR DB!
                }
            }
        }
    }
}
