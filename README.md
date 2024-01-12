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
make build
```

## Run

### Local Testnet

Relay chain repository [Polkadot](https://github.com/paritytech/polkadot) has to be built in `../polkadot` sibling directory
Install `polkadot-launch` utility used to start network.

```
npm install -g polkadot-launch
```

Start local testnet with 4 relay chain validators and Basilisk as a parachain with 2 collators.

```
cd ../rococo-local
polkadot-launch config.json
```

Observe Basilisk logs

```
multitail 99*.log
```

### Interaction with the node

Go to the Polkadot apps at https://dotapps.io

Connect to the local testnet at `ws://localhost:9988` or live `wss://basilisk.hydradx.io:9944`

*NOTE - FixedU128 type is not yet implemented for polkadot apps. Balance is a measure so price can be reasonably selected. If using polkadot apps to create pool:*
- 1 Mega Units equals 1:1 price
- 20 Mega Units equals 20:1 price
- 50 Kilo Units equals 0.05:1 price

### Testing of storage migrations and runtime upgrades

The `try-runtime` tool can be used to test storage migrations and runtime upgrades against state from a real chain.
Run the following command to test against the state on Basilisk on Kusama.
Don't forget to use a runtime built with `try-runtime` feature.
```
try-runtime --runtime ./target/release/wbuild/basilisk-runtime/basilisk_runtime.wasm on-runtime-upgrade --checks all live --uri wss://rpc.basilisk.cloud:443
```
or against the Basilisk testnet on Rococo using `--uri wss://rococo-basilisk-rpc.hydration.dev:443`


### Chopsticks simulations
`Chopsticks` can be used to dry-run any transaction in parallel reality of any Substrate network.

Setting up the repo:
```bash
git clone --recurse-submodules https://github.com/AcalaNetwork/chopsticks.git && cd chopsticks
yarn
yarn build-wasm
```
To run Kusama-Basilisk-Karura setup use configs from `launch-configs/chopsticks` and run
```bash
npx @acala-network/chopsticks@0.3.11 xcm --relaychain=configs/kusama.yml --parachain=configs/basilisk.yml --parachain=configs/karura.yml
```