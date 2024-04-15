set -e

OUTPUT=$(./scenario-generator --steps 200)
SCENARIO_NAME="$(cut -d' ' -f2 <<<"$OUTPUT")"
echo "Using scenario $SCENARIO_NAME"
./scenario-tester --rpc ${RPC} --chain-id ${CHAIN_ID} --faucet-sk ${FAUCET_SK} --scenario scenarios/$SCENARIO_NAME.json