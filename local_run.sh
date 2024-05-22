set -e

CHAIN_ID=${1}
SK=${2}

OUTPUT=$(cargo run --bin scenario-generator -- --steps 200)
SCENARIO_NAME="$(cut -d' ' -f2 <<<"$OUTPUT")"
echo "Using scenario $SCENARIO_NAME"
cargo run --bin scenario-tester -- --rpc 'http://0.0.0.0:27657' --chain-id $CHAIN_ID --faucet-sk ${SK} --scenario scenarios/$SCENARIO_NAME.json --workers 1