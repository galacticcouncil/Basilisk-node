# Fork off chain testing

## Initial setup
* install `docker`, `docker-compose` and build binaries for parachain and relaychain
* check `env` file and set correct paths 

## WARNING
* `PARACHAIN_SPEC` variable in `env` file **MUST BE A FILE PATH** `local` or `dev` will not work.

## Running fork

* *RELAYCHAIN* - `./fork-testing.sh --star-relay` - start `alice, bob, charlie, dave` nodes for relaychain.
* *PARACHAIN* - `./fork-testing.sh --star-para` - start `coll-01, coll-02, coll-03` nodes for parachain.
* *RELAYCHAIN* - wait one relaychain session(2min, 20 blocks) before rest of relaychain actions`
* *PARACHAIN* - wait till parachain is initialized`
* *PARACHAIN* - set collator keys: `./fork-testing.sh --set-collator-keys`. Keys in `./parachain/keys/*.json` are `alice, bob, charlie`
* *PARACHAIN* - generate state and wasm `./fork-testing.sh --get-para-state-wasm`
* *RELAYCHAIN* - transfer some funds to `Alice`(sudo)
* *RELAYCHAIN* - clean up "forked" parachains `sudo->parasSudoWrapper_sudoScheduleParaCleanup(id)`
* *RELAYCHAIN* - register parachain `sudo->parasSudoWrapper_sudoScheduleParaInitialize(id, genesis)`. New parathread will show in UI on next session **!!!WAIT FOR IT**
* *PARACHAIN* - after parachain inclusion it should start produce blocks 
* *RELAYCHAIN* - `./fork-testing.sh --stop-relay`
* *PARACHAIN* - `./fork-testing.sh --stop-para`

### Note
* `sudo->paras->forceQueueAction(para)`- can be used to speed up parachain actions
* chains are always start from 0 if `./fork-testing.sh` is used
* KUSAMA - kusama binary needs modifications, some pallets have to be added. Use: `git clone git@github.com:Phala-Network/polkadot.git` branch: `kusama-para-test-093` 

### Networking
* *RELAYCHAIN* - `alice` - `127.0.0.1:9944`
* *RELAYCHAIN* - `bob` - `127.0.0.1:9955`
* *RELAYCHAIN* - `charlie` - `127.0.0.1:9966`
* *RELAYCHAIN* - `dave` - `127.0.0.1:9977`
* *PARACHAIN* - `coll-01` - `127.0.0.1:1144`, `127.0.0.1:1133`
* *PARACHAIN* - `coll-01` - `127.0.0.1:2244`, `127.0.0.1:2233`
* *PARACHAIN* - `coll-01` - `127.0.0.1:3344`, `127.0.0.1:3333`

## Useful commands
* start/stop parchain and relaychain with one cmd `./fork-testing.sh --start/--stop`
* generate `local` chainspec `../target/release/basilisk build-spec --chain local --raw > local-raw.json`
* view logs relaychain `docker-compose --env-file env -f relaychain.yml logs --tail=100 -f {alice|bob|charlie|dave}` 
* view logs parachain `docker-compose --env-file env -f parachain.yml logs --tail=100 -f {coll-01|coll-02|coll-03}`
