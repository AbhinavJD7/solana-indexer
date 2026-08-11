# Architectural Decisions

### 1. Helius Geyser gRPC vs Standard WebSockets
**Decision:** We use Geyser gRPC (`yellowstone-grpc`) instead of standard RPC WebSockets.
**Rationale:** WebSockets add 100-500ms of latency due to JSON parsing and RPC overhead. Geyser streams raw protobuf data directly from the validator's memory. For an MEV/Sniper bot, this zero-latency edge is the difference between winning and losing a trade.

### 2. Raydium Fee Account Filter vs SPL Token Filter
**Decision:** We filter transactions by `7YttLkHDoNj9wyDur5pM1ejNaAvT9X4eqaYcHQqtj2G5` (Raydium Create Pool Fee) rather than the entire SPL Token program.
**Rationale:** Subscribing to the entire SPL Token program exhausted over 500,000 Helius credits in a single day due to massive network noise. The Fee account is *only* touched when a new pool is created, eliminating 99.9% of the noise and drastically saving RPC costs.

### 3. Modular Architecture vs Monolithic main.rs
**Decision:** Split the codebase into `database.rs`, `grpc.rs`, and `processor.rs`.
**Rationale:** A monolithic `main.rs` became unreadable and impossible to debug. By isolating state (DB), networking (gRPC), and business logic (Processor), we can safely test and modify the sniper logic without breaking the database connections.

### 4. sqlx over ORMs (Prisma/Diesel)
**Decision:** We use `sqlx` to write raw SQL queries for Supabase.
**Rationale:** While ORMs are great for standard web apps, high-frequency indexing requires maximum performance. `sqlx` allows us to write highly optimized, async raw SQL (like `ON CONFLICT DO NOTHING`) without the overhead of an ORM.
