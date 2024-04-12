set -e

OUTPUT=$(cargo run --bin scenario-generator -- --steps 100)
SCENARIO_NAME="$(cut -d' ' -f2 <<<"$OUTPUT")"
echo "Using scenario $SCENARIO_NAME"
sleep 1
cargo run --bin scenario-tester -- --rpc 'https://proxy.heliax.click/internal-devnet-306.053460e56b' --chain-id internal-devnet-306.053460e56b --faucet-sk 00dfd790bd727b708f8b846374c596d886eaf1ebf0fc4394530e0a9b24aa630963 --scenario scenarios/$SCENARIO_NAME.json