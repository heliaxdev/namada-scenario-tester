# Namada Scenario Testing

Create or generate scenarios for namada chains and run them programmatically. 

## How to run a scenario

- `cargo run --bin scenario-tester -- --rpc <tendermint-rpc> --chain-id <chain-id> --faucet-sk <faucet-sk> --scenario <file-path-to-scenario>`
- `cargo run --bin scenario-tester -- --rpc <tendermint-rpc> --chain-id <chain-id> --faucet-sk <faucet-sk>`
    - will select a random sceanario file from the `scenario` folder

## How to generate a scenario

- `cargo run --bin scenario-generator -- --steps <number-of-steps>`