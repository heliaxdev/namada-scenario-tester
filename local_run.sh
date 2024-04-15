set -e

OUTPUT=$(cargo run --bin scenario-generator -- --steps 200)
SCENARIO_NAME="$(cut -d' ' -f2 <<<"$OUTPUT")"
echo "Using scenario $SCENARIO_NAME"
cargo run --bin scenario-tester -- --rpc 'https://proxy.heliax.click/internal-devnet-306.053460e56b' --chain-id internal-devnet-306.053460e56b --faucet-sk t --scenario scenarios/$SCENARIO_NAME.json