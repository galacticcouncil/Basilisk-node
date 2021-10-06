# Basilisk node

## Local Development

Follow these steps to prepare a local Substrate development environment :hammer_and_wrench:

### Simple Setup

Install all the required dependencies with a single command (be patient, this can take up to 30
minutes).

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast
```

### Manual Setup

Find manual setup instructions at the
[Substrate Developer Hub](https://substrate.dev/docs/en/knowledgebase/getting-started/#manual-installation).

### Build

Once the development environment is set up, build the node. This command will build the
[Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution) and
[native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

```bash
cargo build --release
```

## Run

### Rococo local testnet

Relay chain repository (polkadot) has to be built in `../polkadot`
and uses `polkadot-launch` utility that has to be installed from the latest sources.

```
git clone https://github.com/paritytech/polkadot-launch.git
cd polkadot-launch
yarn
yarn build
chmod +x dist/index.js
npm link
```

Starts local testnet with 4 relay chain validators and Basilisk as a parachain.

```
cd ../rococo-local
polkadot-launch config.json
```

Observe Basilisk logs

```
multitail 99*.log
```

### Testing Runtime

There is also an option to run the testing runtime with less restrictive settings to facilitate testing of new features.
The following command starts a dev chain, and the testing runtime is used as a runtime for our node.
```bash
./target/release/basilisk --dev --runtime=testing
```
The testing runtime currently supports only two chain specifications: _dev_ and _local_ testnet.
Both runtimes store blockchain data in the same directories( e.g. the _dev_ directory is shared for both runtimes 
started with the `--dev` parameter. That's why it is important to purge chain data when switching to different runtime( note: `--runtime` parameter can't be used when purging chain data)

In the case of starting a testnet using the `polkadot-launch` tool, 
we don't have an option to communicate to its internal commands that we would like to use the testing runtime.
To overcome this limitation, rename the binary so it starts with the `testing` prefix, e.g. `testing-basilisk`.
Such a binary always uses the testing runtime, even if the `--runtime testing` option is not specified.

### Interaction with the node

Go to the Polkadot apps at https://dotapps.io

Then open settings screen -> developer and paste

*NOTE - FixedU128 type is not yet implemented for polkadot apps. Balance is a measure so price can be reasonably selected. If using polkadot apps to create pool:*
- 1 Mega Units equals 1:1 price
- 20 Mega Units equals 20:1 price
- 50 Kilo Units equals 0.05:1 price

```
{
  "alias":{
    "tokens":{
      "AccountData":"OrmlAccountData"
    }
  },
  "types":[
    {
      "AssetPair":{
        "asset_in":"AssetId",
        "asset_out":"AssetId"
      },
      "Amount":"i128",
      "AmountOf":"Amount",
      "Address":"AccountId",
      "OrmlAccountData":{
        "free":"Balance",
        "frozen":"Balance",
        "reserved":"Balance"
      },
      "Fee":{
        "numerator":"u32",
        "denominator":"u32"
      },
      "BalanceInfo":{
        "amount":"Balance",
        "assetId":"AssetId"
      },
      "Chain":{
        "genesisHash":"Vec<u8>",
        "lastBlockHash":"Vec<u8>"
      },
      "Currency":"AssetId",
      "CurrencyId":"AssetId",
      "CurrencyIdOf":"AssetId",
      "Intention":{
        "who":"AccountId",
        "asset_sell":"AssetId",
        "asset_buy":"AssetId",
        "amount":"Balance",
        "discount":"bool",
        "sell_or_buy":"IntentionType"
      },
      "IntentionId":"Hash",
      "IntentionType":{
        "_enum":[
          "SELL",
          "BUY"
        ]
      },
      "LookupSource":"AccountId",
      "Price":"Balance",
      "ClassId":"u64",
      "TokenId":"u64",
      "ClassData":{
        "is_pool":"bool"
      },
      "TokenData":{
        "locked":"bool"
      },
      "ClassInfo":{
        "metadata":"Vec<u8>",
        "total_issuance":"TokenId",
        "owner":"AccountId",
        "data":"ClassData"
      },
      "TokenInfo":{
        "metadata":"Vec<u8>",
        "owner":"AccountId",
        "data":"TokenData"
      },
      "ClassInfoOf":"ClassInfo",
      "TokenInfoOf":"TokenInfo",
      "ClassIdOf":"ClassId",
      "TokenIdOf":"TokenId",
      "OrderedSet":"Vec<AssetId>",
      "VestingSchedule":{
        "start":"BlockNumber",
        "period":"BlockNumber",
        "period_count":"u32",
        "per_period":"Compact<Balance>"
      },
      "VestingScheduleOf":"VestingSchedule",
      "LBPAssetInfo":{
        "id":"AssetId",
        "amount":"Balance",
        "initial_weight":"LBPWeight",
        "final_weight":"LBPWeight"
      },
      "LBPWeight":"u128",
      "WeightPair":{
        "weight_a":"LBPWeight",
        "weight_b":"LBPWeight"
      },
      "WeightCurveType":{
        "_enum":[
          "Linear"
        ]
      },
      "PoolId":"AccountId",
      "BalanceOf":"Balance",
      "AssetType": {
        "_enum": {
		  "Token": "Null",
		  "PoolShare": "(AssetId,AssetId)"
	    }
      },
      "Pool":{
        "owner":"AccountId",
        "start":"BlockNumber",
        "end":"BlockNumber",
        "assets":"AssetPair",
        "initial_weights":"WeightPair",
        "final_weights":"WeightPair",
        "last_weight_update":"BlockNumber",
        "last_weights":"WeightPair",
        "weight_curve":"WeightCurveType",
        "pausable":"bool",
        "paused":"bool",
        "fee":"Fee",
        "fee_receiver":"AccountId"
      },
      "AssetNativeLocation":"MultiLocation",
      "AssetDetails":{
        "name":"Vec<u8>",
        "asset_type":"AssetType",
        "existential_deposit":"Balance",
        "locked":"bool"
      },
      "AssetDetailsT": "AssetDetails",
      "AssetMetadata":{
        "symbol":"Vec<u8>",
        "decimals":"u8"
      },
      "AssetInstance": "AssetInstanceV0",
      "MultiLocation": "MultiLocationV0",
      "AssetNativeLocation": "MultiLocation",
      "MultiAsset": "MultiAssetV0",
      "Xcm": "XcmV0",
      "XcmOrder": "XcmOrderV0"
    }
  ]
}
```

Connect to the `wss://basilisk.hydradx.io:9944` or local node.

