set -e

CHAIN_ID=${1}
SK=${2}
RPC=${3}

OUTPUT=$(cargo run --bin scenario-generator -- --steps 200)
SCENARIO_NAME="$(cut -d' ' -f2 <<<"$OUTPUT")"
echo "Using scenario $SCENARIO_NAME"
RUST_BACKTRACE=full NAMADA_MASP_PARAMS_DIR="/Users/fraccaman/Library/Application Support/MASPParams" cargo run --bin scenario-tester -- --rpc ${RPC} --chain-id $CHAIN_ID --faucet-sk ${SK} --scenario scenarios/$SCENARIO_NAME.json --workers 1
