set -e

CHAIN_ID=${1}
SK=${2}
RPC=${3}

OUTPUT=$(cargo run --bin scenario-generator -- --steps 30)
SCENARIO_NAME="$(cut -d' ' -f2 <<<"$OUTPUT")"
echo "Using scenario $SCENARIO_NAME"
RUST_BACKTRACE=full cargo run --bin scenario-tester --release -- --rpc ${RPC} --chain-id $CHAIN_ID --faucet-sk ${SK} --scenario scenarios/$SCENARIO_NAME.json --workers 1 --avoid-check
