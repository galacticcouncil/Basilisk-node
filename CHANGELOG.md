# v2.0.0 (Sun Jul 11 2021)

#### 💥 Breaking Change

- fix(xyk)!: create_pool respects min liquidity limit [#96](https://github.com/galacticcouncil/Basilisk-node/pull/96) ([@Roznovjak](https://github.com/Roznovjak))
- feat(multipayment)!: use fallback in set_currency [#93](https://github.com/galacticcouncil/Basilisk-node/pull/93) ([@Roznovjak](https://github.com/Roznovjak))

#### Authors: 1

- Richard Roznovjak ([@Roznovjak](https://github.com/Roznovjak))

---

# v1.0.0 (Tue Jul 06 2021)

#### 💥 Breaking Change

- chore!: add hydradx pallets [#73](https://github.com/galacticcouncil/Basilisk-node/pull/73) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- chore!: update hydra dependencies - use latest pallets [#62](https://github.com/galacticcouncil/Basilisk-node/pull/62) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- feat!: add Basilisk ss58 [#47](https://github.com/galacticcouncil/Basilisk-node/pull/47) ([@jak-pan](https://github.com/jak-pan) [@lumir-mrkva](https://github.com/lumir-mrkva) [@enthusiastmartin](https://github.com/enthusiastmartin))

#### 🚀 Enhancement

- feat: add trading limits to xyk pool [#91](https://github.com/galacticcouncil/Basilisk-node/pull/91) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- feat: vesting [#71](https://github.com/galacticcouncil/Basilisk-node/pull/71) ([@green-jay](https://github.com/green-jay) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: for off testing [#79](https://github.com/galacticcouncil/Basilisk-node/pull/79) ([@martinfridrich](https://github.com/martinfridrich) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: NFT [#13](https://github.com/galacticcouncil/Basilisk-node/pull/13) ([@green-jay](https://github.com/green-jay) [@enthusiastmartin](https://github.com/enthusiastmartin))
- feat: update fee calculation [#66](https://github.com/galacticcouncil/Basilisk-node/pull/66) ([@Roznovjak](https://github.com/Roznovjak) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: collator set [#49](https://github.com/galacticcouncil/Basilisk-node/pull/49) ([@martinfridrich](https://github.com/martinfridrich) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: utility [#59](https://github.com/galacticcouncil/Basilisk-node/pull/59) ([@green-jay](https://github.com/green-jay))
- feat: vesting [#51](https://github.com/galacticcouncil/Basilisk-node/pull/51) ([@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: initial chain setup [#36](https://github.com/galacticcouncil/Basilisk-node/pull/36) ([@martinfridrich](https://github.com/martinfridrich) [@lumir-mrkva](https://github.com/lumir-mrkva) [@jak-pan](https://github.com/jak-pan))
- feat: add extrinsics filter [#44](https://github.com/galacticcouncil/Basilisk-node/pull/44) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: build script utils [#9](https://github.com/galacticcouncil/Basilisk-node/pull/9) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: check rococo local binaries ([@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: hydradx ss58 ([@lumir-mrkva](https://github.com/lumir-mrkva))

#### 🐛 Bug Fix

- chore: v0.9.7 upgrade [#87](https://github.com/galacticcouncil/Basilisk-node/pull/87) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@martinfridrich](https://github.com/martinfridrich) [@jak-pan](https://github.com/jak-pan))
- chore: pallet-utility dependency fix [#92](https://github.com/galacticcouncil/Basilisk-node/pull/92) ([@jak-pan](https://github.com/jak-pan))
- chore: use latest math crate [#90](https://github.com/galacticcouncil/Basilisk-node/pull/90) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- test(exchange): add missing tests [#88](https://github.com/galacticcouncil/Basilisk-node/pull/88) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@Roznovjak](https://github.com/Roznovjak))
- test(xyk): add missing tests [#85](https://github.com/galacticcouncil/Basilisk-node/pull/85) ([@Roznovjak](https://github.com/Roznovjak) [@enthusiastmartin](https://github.com/enthusiastmartin))
- chore: update rust-toolchain [#86](https://github.com/galacticcouncil/Basilisk-node/pull/86) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- ci: run workflow on push to master [#84](https://github.com/galacticcouncil/Basilisk-node/pull/84) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- chore: use latest math package [#83](https://github.com/galacticcouncil/Basilisk-node/pull/83) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- ci: add code test coverage [#82](https://github.com/galacticcouncil/Basilisk-node/pull/82) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- fix: cargo update [#72](https://github.com/galacticcouncil/Basilisk-node/pull/72) ([@martinfridrich](https://github.com/martinfridrich))
- chore: release candidate 0.1.0 [#65](https://github.com/galacticcouncil/Basilisk-node/pull/65) ([@lumir-mrkva](https://github.com/lumir-mrkva))
- ci: fix inclusion test [#67](https://github.com/galacticcouncil/Basilisk-node/pull/67) ([@lumir-mrkva](https://github.com/lumir-mrkva))
- chore: update multi-payment weights due to recent change in the pallet [#68](https://github.com/galacticcouncil/Basilisk-node/pull/68) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- feat: LBP pallet [#11](https://github.com/galacticcouncil/Basilisk-node/pull/11) ([@Roznovjak](https://github.com/Roznovjak) [@martinfridrich](https://github.com/martinfridrich) [@lumir-mrkva](https://github.com/lumir-mrkva) [@enthusiastmartin](https://github.com/enthusiastmartin))
- refactor(nft): remove unnecessary currencybalance from pallet config [#64](https://github.com/galacticcouncil/Basilisk-node/pull/64) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- fix: polkadot launch config & inclusion test [#58](https://github.com/galacticcouncil/Basilisk-node/pull/58) ([@lumir-mrkva](https://github.com/lumir-mrkva))
- rc0 [#57](https://github.com/galacticcouncil/Basilisk-node/pull/57) ([@lumir-mrkva](https://github.com/lumir-mrkva))
- chore: benchmark all pallets in basilisk [#48](https://github.com/galacticcouncil/Basilisk-node/pull/48) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- fix: ss58 use forked substrate [#56](https://github.com/galacticcouncil/Basilisk-node/pull/56) ([@lumir-mrkva](https://github.com/lumir-mrkva) [@enthusiastmartin](https://github.com/enthusiastmartin))
- chore: upgrade 0.9.4 [#55](https://github.com/galacticcouncil/Basilisk-node/pull/55) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- chore: upgrade to polkadot-v0.9.3 [#46](https://github.com/galacticcouncil/Basilisk-node/pull/46) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- ci: test inclusion fixes [#41](https://github.com/galacticcouncil/Basilisk-node/pull/41) ([@lumir-mrkva](https://github.com/lumir-mrkva))
- chore: pin orml to specific revision [#39](https://github.com/galacticcouncil/Basilisk-node/pull/39) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- ci: smoke test [#32](https://github.com/galacticcouncil/Basilisk-node/pull/32) ([@lumir-mrkva](https://github.com/lumir-mrkva))
- chore: update dependecies [#29](https://github.com/galacticcouncil/Basilisk-node/pull/29) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- fix: fix build with runtime benchmarks features ([@enthusiastmartin](https://github.com/enthusiastmartin))
- ci: fork builds [#4](https://github.com/galacticcouncil/Basilisk-node/pull/4) ([@lumir-mrkva](https://github.com/lumir-mrkva) [@enthusiastmartin](https://github.com/enthusiastmartin))
- refactor: use pallets from hydradx-node repo [#3](https://github.com/galacticcouncil/Basilisk-node/pull/3) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- Basilic runner [#5](https://github.com/galacticcouncil/Basilisk-node/pull/5) ([@fakirAyoub](https://github.com/fakirAyoub) [@enthusiastmartin](https://github.com/enthusiastmartin))
- fix: amm tests event checking ([@lumir-mrkva](https://github.com/lumir-mrkva))

#### Authors: 7

- [@green-jay](https://github.com/green-jay)
- [@lumir-mrkva](https://github.com/lumir-mrkva)
- Ayoub Fakir ([@fakirAyoub](https://github.com/fakirAyoub))
- Jakub Pánik ([@jak-pan](https://github.com/jak-pan))
- martin fridrich ([@martinfridrich](https://github.com/martinfridrich))
- Martin Hloska ([@enthusiastmartin](https://github.com/enthusiastmartin))
- Richard Roznovjak ([@Roznovjak](https://github.com/Roznovjak))
