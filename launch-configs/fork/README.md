starts zombienet instance with forked basilisk state downloaded from either:
- rpc endpoint in `STATE_RPC` (defaults to `wss://rpc.basilisk.cloud`) and block `STATE_BLOCK` (latest finalized by default)
- url of json defined in `STATE_SOURCE`

para_id: 2090, relay: rococo-local (kusama-local was retired from the polkadot binary; the local relay is cosmetic for a fork test)

### run with docker

```
docker run -d -p 9988:9988 galacticcouncil/fork:basilisk
```

### run locally

- node >18 required
- you have to have all binaries present on correct paths in `config.json`

```
npm i && npm start
```

### run with live PROD chainspec

- It scrapes the chainspec from the latest finalized block of basilisk mainnet, then spins up the fork via zombienet

```
npm run start:live
```

### run with custom chainspec

If you want to run fork with a custom chainspec, do the following:

1. Save a chainspec via the scraper (basilisk reuses hydration-node's `scraper` binary).
2. Place it at `./data/state.json`.
3. Run `npm run start:raw`.

### preauthorize a runtime upgrade

Set `AUTHORIZE_UPGRADE_CODE_HASH=0x<32-byte wasm hash>` in the container env. The fork
comes up with `System.AuthorizedUpgrade` populated, so a single
`system.applyAuthorizedUpgrade(code)` call enacts the new wasm — no governance step.

Pass `AUTHORIZE_UPGRADE_CHECK_VERSION=false` to skip the spec-version check (default: true).
