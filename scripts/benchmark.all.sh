#!/bin/bash

pallets=("pallet-xyk:xyk"
"pallet-lbp:lbp"
"pallet-nft:nft"
"pallet-asset-registry:registry"
"pallet-xyk-liquidity-mining:xyk_liquidity_mining"
"pallet-transaction-pause:transaction_pause"
"frame-system:system"
"pallet-balances:balances"
"pallet-collator-selection:collator_selection"
"pallet-timestamp:timestamp"
"pallet-democracy:democracy"
"pallet-treasury:treasury"
"pallet-scheduler:scheduler"
"pallet-utility:utility"
"pallet-tips:tips"
"pallet-xcm:xcm"
"cumulus-pallet-xcmp-queue:xcmp_queue"
"pallet-currencies:currencies"
"orml-tokens:tokens"
"orml-vesting:vesting"
"pallet-duster:duster"
"pallet-transaction-multi-payment:multi_payment"
"pallet-route-executor:route_executor"
"pallet-marketplace:marketplace"
)

command="cargo run --release --features=runtime-benchmarks -- benchmark pallet --pallet=[pallet] --chain=dev --extrinsic='*' --steps=5 --repeat=20 --output [output].rs --template .maintain/pallet-weight-template.hbs"

for string in "${pallets[@]}"; do

  IFS=':' read -ra subvalues <<< "$string"

  pallet="${subvalues[0]}"
  output="${subvalues[1]}"

  echo "Running benchmark for ${pallet}"

  replaced_command="${command/\[pallet\]/$pallet}"
  replaced_command="${replaced_command/\[output\]/$output}"

  eval "$replaced_command"
done
