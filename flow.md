# Execution Flow

This document traces the exact path a transaction takes from the Solana blockchain into our Sensing Engine.

### 1. Initialization (`src/main.rs`)
- **`main()`** starts the Tokio async runtime.
- Calls **`database::setup_database()`** -> Connects to Supabase and ensures the `transactions` table exists.
- Calls **`grpc::setup_geyser_client()`** -> Establishes the TLS connection to the Helius Geyser node.
- Calls **`grpc::build_raydium_filter()`** -> Constructs the subscription request targeting the Raydium Fee account.

### 2. The Ingestion Loop (`src/main.rs`)
- **`stream.next().await`** -> An infinite loop that waits silently for the Geyser node to push data to us.
- When a message arrives, it matches against `UpdateOneof::Transaction`.

### 3. The Processing Engine (`src/processor.rs`)
- Execution is handed off to **`processor::process_transaction(tx_info, slot)`**.
- Iterates through `meta.log_messages`.
- **Condition:** If the logs contain `"initialize2"`, it flags the transaction as a Raydium Pool Launch.
- **Extraction:** Loops through `message.account_keys` to extract and print the Token Mints involved in the pool creation.
- *(Future: This is where we will trigger the buy execution or database save).*
