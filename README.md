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

Relay chain repository (polkadot) has to be built in `../polkadot`
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

#### Use Testing Runtime

There is also an option to run the testing runtime with less restrictive settings to facilitate testing of new features.
The following command starts a dev node collator, and the testing runtime is used as a runtime for our node.
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

Start local testnet with testing runtime
```
cd ../rococo-local
polkadot-launch testing-config.json
```

### Interaction with the node

Go to the Polkadot apps at https://dotapps.io

Connect to the local testnet at `ws://localhost:9988` or live `wss://basilisk.hydradx.io:9944`

*NOTE - FixedU128 type is not yet implemented for polkadot apps. Balance is a measure so price can be reasonably selected. If using polkadot apps to create pool:*
- 1 Mega Units equals 1:1 price
- 20 Mega Units equals 20:1 price
- 50 Kilo Units equals 0.05:1 price

