# Solana Indexer TODOs

## Future Infrastructure Goals (Path 1)
Eventually, we want to run a local Solana node with the Geyser plugin to simulate a professional enterprise environment for free.

**Steps to complete later:**
- [ ] Install the Solana CLI (`solana-test-validator`).
- [ ] Clone the `rpcpool/yellowstone-grpc` repository or download the pre-built binary for the `yellowstone-grpc-geyser` plugin.
- [ ] Ensure the Rust toolchain matches the Solana CLI version.
- [ ] Create a `config.json` file pointing to the compiled plugin (the `.so` or `.dylib` file) and set the gRPC port (e.g., `0.0.0.0:10000`).
- [ ] Start the local validator with the plugin: `solana-test-validator --geyser-plugin-config config.json`
- [ ] Update the indexer code to connect to `http://127.0.0.1:10000` without an auth token.


### Ideas in which I can follow later

Can I ask my user to enter the wallet address at home page and create a separate database collection for that user to get the record to that wallet address txn there

