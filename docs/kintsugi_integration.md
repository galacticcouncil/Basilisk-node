# Kintsugi integration

Steps to execute cross-chain transfers of KBTC:

### Setup a local testnet

- Relaychain: v0.9.16
- Basilisk: <https://github.com/galacticcouncil/Basilisk-node/tree/60bfb4f5509758c945fcf81c9b435a4474575cd8>
- Interlay: <https://github.com/interlay/interbtc/tree/ba26ef897b2ed93d881786946aff9626b37f90fa>

The following changes are needed on the Kintsugi node
```diff
diff --git a/parachain/runtime/testnet/src/lib.rs b/parachain/runtime/testnet/src/lib.rs
index b4d253ff..8821a5eb 100644
--- a/parachain/runtime/testnet/src/lib.rs
+++ b/parachain/runtime/testnet/src/lib.rs
@@ -1309,7 +1309,7 @@ construct_runtime! {
 
         // XCM helpers.
         XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 40,
-        PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin} = 41,
+        PolkadotXcm: pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin, Config} = 41,
         CumulusXcm: cumulus_pallet_xcm::{Pallet, Call, Event<T>, Origin} = 42,
         DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 43,
 
diff --git a/parachain/src/chain_spec.rs b/parachain/src/chain_spec.rs
index 5d28b609..063aa5d1 100644
--- a/parachain/src/chain_spec.rs
+++ b/parachain/src/chain_spec.rs
@@ -586,6 +586,7 @@ fn testnet_genesis(
             start_height: testnet_runtime::YEARS * 4,
             inflation: FixedU128::checked_from_rational(2, 100).unwrap(), // 2%
         },
+        polkadot_xcm: Default::default()
     }
 }
```

### Polkadot-launch config

<https://github.com/galacticcouncil/Basilisk-node/blob/master/rococo-local/basilisk-kintsugi.json>

### Transactions

- Register KBTC asset on Basilisk node
  - 0x28000500104b425443000a000000000000000000000000000000
- Register KBTC location on Basilisk node
  - 0x2800050303000000010200411f0608000b
- Set KBTC balance on Kintsugi testnet node
  - 0x02000904d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d000b13000064a7b3b6e00d00
- Transfer KBTC from Kintsugi to Basilisk
  - 0x2c00000b0000b2d3595bf006000000000000000001010200a9200100d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d0046c32300000000
- Transfer KBTC from Basilisk to Kintsugi
  - 0x12000300000000008a5d78456301000000000000000001010200411f01008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a480000c16ff2862300
