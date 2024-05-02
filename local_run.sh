set -e

OUTPUT=$(cargo run --bin scenario-generator -- --steps 200)
SCENARIO_NAME="$(cut -d' ' -f2 <<<"$OUTPUT")"
echo "Using scenario $SCENARIO_NAME"
cargo run --bin scenario-tester -- --rpc 'http://0.0.0.0:27657' --chain-id local.d68bc8c598bbd77f58e5230f --faucet-sk 000d5e9d7d66f0e4307edacde6e6578e31d331bcf234352647d00d20955102d3ce --scenario scenarios/$SCENARIO_NAME.json