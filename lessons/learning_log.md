# Solana Indexer Learning Log

## Day 1: Real-Time Data Ingestion with Geyser gRPC

**Core Concepts Learned:**
- **RPC vs. Geyser:** Standard RPC polling (`getProgramAccounts`) is too slow and heavily rate-limited for real-time indexing. The **Geyser Plugin** sits directly inside the Solana validator, intercepting state changes instantly and pushing them out over a high-speed gRPC stream.
- **Infrastructure Providers:** Learned that providers like Helius (using their LaserStream architecture) and Triton One run the open-source Yellowstone Geyser plugin on their nodes.

**Technical Implementation:**
- **gRPC Client Setup:** Successfully connected to the Helius Devnet Geyser endpoint (`laserstream-devnet-ewr.helius-rpc.com`) using `yellowstone-grpc-client`.
- **Authentication & TLS:** Handled gRPC authentication by injecting the API key via the `x-token` header (since gRPC can drop URL queries). Configured `tonic` to use `ClientTlsConfig` with `webpki_roots` to securely connect over HTTPS.
- **Subscription Filters:** Created a `SubscribeRequestFilterBlocks` to filter the data firehose. Learned how to exclude heavy payloads (`include_transactions: false`) and disable the Cuckoo filter (`cuckoo_account_include: None`) to receive lightweight slot updates.
- **Commitment Levels:** Set `commitment: Some(1)` (Confirmed) to avoid indexing ephemeral forks, and handled the `from_slot: None` parameter to ensure streaming starts exactly at the live tip of the blockchain.
- **Async Streaming:** Utilized `tokio` and `futures::stream::StreamExt` to establish an infinite listener loop that pattern matches on incoming `UpdateOneof::Block` and `UpdateOneof::Ping` events in real-time.

---

## Day 2: Decoding Raw Transactions and Personal Tracking

**Core Concepts Learned:**
- **Targeted Subscriptions:** Transitioned from subscribing to empty blocks to subscribing to specific transactions using `SubscribeRequestFilterTransactions`. 
- **Program & Wallet Tracking:** Learned to pass an `account_include` array to track all traffic hitting the SPL Token Program, and successfully tested it by tracking a personal wallet address in real-time.
- **Data De-serialization Strategy:** Instead of manually deserializing a blob of bytes using `bincode` and `solana-sdk`, discovered that the Yellowstone gRPC stream provides a heavily structured Protobuf object (`SubscribeUpdateTransaction`), making field extraction simple.

**Technical Implementation:**
- **Signature Extraction:** Grabbed the raw 64-byte `signature` array from the `tx_info` object and utilized the `bs58` crate to encode it into a standard Base58 string. Used this string to dynamically construct a functional Solscan explorer link.
- **Message Parsing:** Navigated into `tx_info.transaction.message` to extract the `instructions.len()` and the `account_keys` array, successfully converting the raw pubkeys into readable Base58 format.
- **Balance Calculations (Lamports to SOL):** Accessed the `tx_info.meta` field to retrieve `pre_balances` and `post_balances` arrays. Learned that these arrays map 1-to-1 with the `account_keys` index. Calculated the financial difference by casting to `i64` and converting Lamports to SOL (dividing by 1,000,000,000).

**Status:**
Successfully tracked personal Devnet wallet activity and extracted deep financial metadata from the SPL Token Program! 🚀

---

## Day 3: Cloud Database Integration & Data Pipelining

**Core Concepts Learned:**
- **Asynchronous Data Pipelines:** When streaming high-speed blockchain data, any blocking operation (like writing to a database) can cause the gRPC stream to lag or drop data. Learned to push database inserts into background threads using `tokio::spawn` to keep the ingestion loop running at max speed.
- **Connection Poolers:** Explored how cloud databases like Supabase use PgBouncer to manage high-volume connection requests. Discovered the critical difference between **Transaction Mode** (which does not support prepared statements easily) and **Session Mode** (which perfectly maps to typical backend connections).

**Technical Implementation:**
- **sqlx Configuration:** Configured `sqlx::PgPoolOptions` to create a robust connection pool to the Supabase Postgres instance. Overcame compile-time macro issues by correctly utilizing the Session Mode pooler port (`5432`).
- **Automated Schema Generation:** Leveraged `sqlx::query` to execute a `CREATE TABLE IF NOT EXISTS` statement during startup to programmatically prepare the data structures (`transactions`).
- **Data Pruning:** Implemented logic to only save accounts that actually experienced a `change_sol != 0.0`. This aggressively prunes noise (like read-only data accounts or untouched programs) before insertion, saving massive amounts of database storage.
- **Background Inserts:** Constructed vectors for `accounts_involved` and `balance_changes`, cloned the DB connection pool, and used `sqlx::query` chained with `.bind()` inside a `tokio::spawn(async move { ... })` task to gracefully insert the structured blockchain data entirely in the background.

**Status:**
Successfully deployed a fully autonomous Web3 data pipeline pushing real-time Solana transactions straight to a cloud database! 🚀

---

## Day 4: Advanced Architecture & The MEV Sensing Engine

**Core Concepts Learned:**
- **Modular Rust Architecture:** Learned how to break a monolithic `main.rs` file into independent modules (`database.rs`, `grpc.rs`, `processor.rs`). Discovered that making a function `pub` is required to share logic across the workspace, creating a clean, production-grade orchestrator in `main.rs`.
- **gRPC vs WebSockets:** Deep dived into the latency differences between RPC-based WebSockets and direct-validator Geyser gRPC streams. Understood why MEV bots and high-frequency trading firms exclusively use Geyser for zero-latency data interception.
- **RPC Credit Optimization:** Realized that subscribing to massive programs (like the entire SPL Token program) drains Helius credits rapidly. Learned how to find "Alpha" filter accounts (like the Raydium Create Pool Fee account: `7Ytt...`) to filter out 99.9% of network noise and only stream highly specific events.

**Technical Implementation:**
- **Refactoring:** Successfully split the codebase into logical domains, separating database connection logic, gRPC subscription filters, and transaction parsing into dedicated files.
- **Log Parsing:** Wrote a processor engine that iterates over `meta.log_messages` in real-time, scanning raw contract outputs for the exact `initialize2` string emitted by Raydium AMM V4 during a token launch.
- **Devnet Simulation:** Validated the sniper's sensing engine by executing a microscopic `solana transfer` via the CLI, appending a string memo to trick the indexer into successfully triggering the token launch alarm.

**Status:**
Successfully transformed a generic blockchain indexer into a highly-specialized, zero-latency MEV Sensing Engine capable of detecting liquidity pools the millisecond they launch! 🎯

---

## Day 5: The Full MEV Pipeline & Analytics

**Core Concepts Learned:**
- **Asynchronous Concurrency:** Learned how to use `tokio::spawn` to offload heavy RPC network calls to background threads, ensuring the primary gRPC stream never blocks or falls behind the blockchain.
- **Graceful Error Handling:** Handled the difference between real token launches and simulated launches by writing defensive logic (e.g., defaulting to "Unknown Token" when account keys are missing).
- **Secondary RPC Verification:** Integrated `solana-client` to reach out to the standard RPC network to inspect Mint Accounts for `mint_authority` status (Rug Pull Check).

**Technical Implementation:**
- Modified `main.rs` to pass a cloned PostgreSQL connection pool into the Tokio background threads.
- Built a bash-based `simulator.sh` spam bot to rapidly fire transactions at the Devnet, successfully stress-testing our processing engine.
- Re-wired the database to a specialized `raydium_pools` table in Supabase, capturing Token Mints, Names, Symbols, and Safety ratings.

**Status:**
We now have a professional indexer that not only detects pools at zero-latency but actively scans them for Rug Pulls and saves the data to a cloud analytics database!


