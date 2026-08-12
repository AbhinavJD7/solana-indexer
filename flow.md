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
- **Thread Handoff:** The main loop clones the database pool and transaction data, then uses `tokio::spawn` to throw the heavy processing onto a background thread so the main loop never blocks.

### 3. The Processing Engine (`src/processor.rs`)
- Background thread begins execution in **`processor::process_transaction(tx_info, slot, db_pool)`**.
- Iterates through `meta.log_messages` to check for `"initialize2"`.
- **RPC Safety Check:** If it's a real launch, makes a secondary HTTP request to the standard Solana RPC to verify the Mint Authority is revoked.
- **Database Insert:** Safely writes the final Token Mint, Name, Symbol, and Safety rating to the Supabase `raydium_pools` table.
