# Namada Sceneraio Testing

Create scenarios for namada chains and run them programmatically.

## How to run

`cargo run -- --cargo-env <development|production> --scenario scenarios/<file>.json --rpcs <ip:port|http> --chain-id <chain-id>`

## Available task

- Tasks
    - wallet-new-key
    - tx-init-account
    - tx-transparent-transfer
- Checks
    - check-balance
    - check-tx
- Waits
    - wait-epoch
    - wait-height
- Queries
    - query-account
    - query-balance