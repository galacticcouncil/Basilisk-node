# pallet-stableswap

## Stableswap pallet (v1)

Curve/stableswap AMM implementation.

Version v1 - supports only 2 assets per pool.

#### Terminology

* **LP** - liquidity provider
* **Share Token** - a token representing share asset of specific pool. Each pool has its own share token.
* **Amplification** - curve AMM pool amplification parameter

### Assumptions

Only 2 assets pool are possible to create in V1.

A pool can be created only by allowed `CreatePoolOrigin`.

LP must add liquidity of both pool assets. in V1 it is not allowed single token LPing.

LP specifies an amount of liquidity to be added of one selected asset, the required amount of second pool asset is calculated
in a way that the ratio does not change.

LP is given certain amount of shares by minting a pool's share token.

When LP decides to withdraw liquidity, it receives both assets. Single token withdrawal is not supported.

### Interface

#### Dispatchable functions

* `create_pool`
* `add_liquidity`
* `remove_liquidity`
* `sell`
* `buy`


License: Apache 2.0
