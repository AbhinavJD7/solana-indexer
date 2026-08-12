# Solana MEV Indexer & Raydium Sniper

A production-grade, zero-latency Solana blockchain indexer built in Rust. This sensing engine connects directly to a validator's Geyser gRPC stream to detect new Raydium Liquidity Pools the millisecond they launch.

It features asynchronous multi-threading, real-time safety checks (Rug Pull detection), and autonomous PostgreSQL storage.

## Architecture
- **Language:** Rust
- **Data Stream:** Helius Geyser gRPC (`yellowstone-grpc`)
- **Concurrency:** Tokio Async Threads (`tokio::spawn`)
- **Database:** PostgreSQL (via `sqlx` & Supabase)
- **RPC:** Standard `solana-client` for secondary Mint Authority verification

## How to Run & Test locally

We included a `simulator.sh` script so you can test the zero-latency sensing engine on Devnet without spending real SOL or waiting for actual pool launches.

### 1. Environment Setup
Create a `.env` file in the root directory and add your Supabase (PostgreSQL) connection string:
```env
DATABASE_URL=postgres://[user]:[password]@[host]:[port]/[db_name]
```

### 2. Start the Indexer
Ensure you have Rust installed, then compile and run the indexer. The first run will take a moment to compile the Solana SDK crates.
```bash
cargo run
```
*(The indexer will connect to Supabase, create the `raydium_pools` table if it doesn't exist, and listen for Devnet blocks).*

### 3. Fire the Simulator!
Open a **second terminal window** and run the simulator script. This will use the Solana CLI to rapidly fire fake Raydium pool launches at the network.
```bash
chmod +x simulator.sh
./simulator.sh
```

### 4. Watch the Magic
Switch back to your first terminal window. You will see the indexer instantly detect the simulated launches, bypass the RPC safety checks (since it's Devnet mock data), and save the analytics directly to your Supabase database!

---
*Note: To run on Mainnet, update the Geyser endpoint in `src/main.rs` to your Mainnet gRPC URL, and the indexer will automatically begin fetching real Token Metadata and executing Rug Pull checks.*
