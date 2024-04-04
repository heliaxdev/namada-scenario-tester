OUTPUT=$(cargo run --bin scenario-generator -- --steps 50)
SCENARIO_NAME="$(cut -d' ' -f2 <<<"$OUTPUT")"
echo "Using scenario $SCENARIO_NAME"
cargo run --bin scenario-tester -- --rpc http://127.0.0.1:27657 --chain-id local.a57599b80494adcad72d392d --faucet-sk 00d19e226c0e7d123d79f5908b5948d4c461b66a5f8aa95600c28b55ab6f5dc772 --scenario scenarios/$SCENARIO_NAME.json