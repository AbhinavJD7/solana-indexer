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

**Status:**
Successfully streamed live Devnet blocks directly into the local terminal! 🚀
