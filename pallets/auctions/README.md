# Auctions pallet

## Description
The Auctions pallet provides extendable auction functionality for NFTs.
The pallet implements an NftAuction trait which allows users to extend the pallet by implementing other
auction types. All auction types must implement bid() and close() functions at their interface.

The auction types share the same store called Auctions. Auction types are represented in a struct which holds
two other structs with general_data (e.g. auction name, start, end) and specific_data for the given auction type.
Besides Auctions, there are are two other stores: NextAuctionId and AuctionOwnerById.

## Dispatchable Functions
- `create` - create an auction

- `update` - update an auction

- `destroy` - destroy an auction

- `bid` - place a bid on an auction

- `close` - close an auction after the end time has lapsed. Not done in a hook for better chain performance.

## Implemented Auction types

### EnglishAuction

In an English auction, participants place bids in a running auction. Once the auction has reached its end time,
the highest bid wins.

The implementation of English auction allows sellers to set a starting price for the object, under which it will not
be sold (auction.general_data.next_bid_min).

It also extens the end time of the auction for any last-minute bids in order to prevent auction sniping.


## Preparing environment

Follow these steps to prepare a local Substrate development environment:

### Simple Setup

Install all the required dependencies with a single command (be patient, this can take up to 30
minutes).

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast
```

### Manual Setup

Find manual setup instructions at the
[Substrate Developer Hub](https://substrate.dev/docs/en/knowledgebase/getting-started/#manual-installation).

### Test
```bash
cargo test --release -p pallet-auctions
```

### Build

Once the development environment is set up, build the node. This command will build the
[Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution) and
[native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

```bash
cargo build --release
```

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
```
Open https://polkadot.js.org/apps/ in the browser and connect to the `wss://basilisk.hydradx.io:9944` or the local node.
```
