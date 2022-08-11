# pallet-stableswap

## Stableswap pallet (v1)

Curve/stableswap AMM implementation.

Supports pools with maximum 5 assets.

#### Terminology

* **LP** - liquidity provider
* **Share Token** - a token representing share asset of specific pool. Each pool has its own share token.
* **Amplification** - curve AMM pool amplification parameter

### Assumptions

A pool can be created only by allowed `CreatePoolOrigin`.


License: Apache 2.0
