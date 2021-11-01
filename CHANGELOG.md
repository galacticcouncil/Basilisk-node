# v5.0.2 (Mon Nov 01 2021)

#### 🐛 Bug Fix

- chore: additional audit tests [#199](https://github.com/galacticcouncil/Basilisk-node/pull/199) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@Roznovjak](https://github.com/Roznovjak))
- feat: XYK and LBP rpc to retrieve pool account id [#213](https://github.com/galacticcouncil/Basilisk-node/pull/213) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- test(exchange): resolve todos [#207](https://github.com/galacticcouncil/Basilisk-node/pull/207) ([@Roznovjak](https://github.com/Roznovjak) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: implement orml benchmarks [#190](https://github.com/galacticcouncil/Basilisk-node/pull/190) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- chore: upgrade v0.9.11 [#202](https://github.com/galacticcouncil/Basilisk-node/pull/202) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@Roznovjak](https://github.com/Roznovjak) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: use system blocknum provider in lbp [#211](https://github.com/galacticcouncil/Basilisk-node/pull/211) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- chore: bump xyk and exchange crate versions [#188](https://github.com/galacticcouncil/Basilisk-node/pull/188) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- chore!: lbp refactor [#185](https://github.com/galacticcouncil/Basilisk-node/pull/185) ([@jak-pan](https://github.com/jak-pan) [@Roznovjak](https://github.com/Roznovjak) [@lumir-mrkva](https://github.com/lumir-mrkva))
- refactor: rename BSX to UNITS [#196](https://github.com/galacticcouncil/Basilisk-node/pull/196) ([@Roznovjak](https://github.com/Roznovjak) [@jak-pan](https://github.com/jak-pan))
- fix!: XCM filter [#200](https://github.com/galacticcouncil/Basilisk-node/pull/200) ([@jak-pan](https://github.com/jak-pan) [@lumir-mrkva](https://github.com/lumir-mrkva))
- fix: use Everything as a call filter [#192](https://github.com/galacticcouncil/Basilisk-node/pull/192) ([@Roznovjak](https://github.com/Roznovjak) [@jak-pan](https://github.com/jak-pan))
- fix: revert orml-token accounts in dev chain spec [#191](https://github.com/galacticcouncil/Basilisk-node/pull/191) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@jak-pan](https://github.com/jak-pan))
- fix: Remove duplicated key AssetNativeLocation [#198](https://github.com/galacticcouncil/Basilisk-node/pull/198) ([@mckrava](https://github.com/mckrava))
- chore: makefile linking fix [#197](https://github.com/galacticcouncil/Basilisk-node/pull/197) ([@Roznovjak](https://github.com/Roznovjak))
- fix: LBP benchmarks [#195](https://github.com/galacticcouncil/Basilisk-node/pull/195) ([@Roznovjak](https://github.com/Roznovjak))

#### ⚠️ Pushed to `master`

- spec version parity restored ([@lumir-mrkva](https://github.com/lumir-mrkva))

#### Authors: 5

- [@lumir-mrkva](https://github.com/lumir-mrkva)
- Jakub Pánik ([@jak-pan](https://github.com/jak-pan))
- Martin Hloska ([@enthusiastmartin](https://github.com/enthusiastmartin))
- Max Kravchuk ([@mckrava](https://github.com/mckrava))
- Richard Roznovjak ([@Roznovjak](https://github.com/Roznovjak))

---

# v5.0.1 (Thu Oct 14 2021)

- fix: enabled XCM

# v5.0.0 (Fri Oct 08 2021)

#### 🚀 Enhancement

- feat: update xyk/exchange events with pool id [#187](https://github.com/galacticcouncil/Basilisk-node/pull/187) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- feat: create testing runtime [#171](https://github.com/galacticcouncil/Basilisk-node/pull/171) ([@Roznovjak](https://github.com/Roznovjak) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: create makefile [#183](https://github.com/galacticcouncil/Basilisk-node/pull/183) ([@Roznovjak](https://github.com/Roznovjak) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat(lbp): add simulation test [#160](https://github.com/galacticcouncil/Basilisk-node/pull/160) ([@Roznovjak](https://github.com/Roznovjak) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: try-runtime integration [#151](https://github.com/galacticcouncil/Basilisk-node/pull/151) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: access amm constants from exchange api [#95](https://github.com/galacticcouncil/Basilisk-node/pull/95) ([@Roznovjak](https://github.com/Roznovjak) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: Integration tests - xcm transfers [#161](https://github.com/galacticcouncil/Basilisk-node/pull/161) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: update xyk events [#150](https://github.com/galacticcouncil/Basilisk-node/pull/150) ([@Roznovjak](https://github.com/Roznovjak))

#### 🐛 Bug Fix

- ci: fix inclusion test [#167](https://github.com/galacticcouncil/Basilisk-node/pull/167) ([@lumir-mrkva](https://github.com/lumir-mrkva))
- chore: update README - xcm v1 types [#184](https://github.com/galacticcouncil/Basilisk-node/pull/184) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- fix: update orml-tokens whitelist accounts [#175](https://github.com/galacticcouncil/Basilisk-node/pull/175) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- chore: upgrade to polkadot-v0.9.10 [#178](https://github.com/galacticcouncil/Basilisk-node/pull/178) ([@green-jay](https://github.com/green-jay) [@enthusiastmartin](https://github.com/enthusiastmartin))
- fix: set ED to max value for unsupported assets [#177](https://github.com/galacticcouncil/Basilisk-node/pull/177) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- fix: update asset details type in readme [#174](https://github.com/galacticcouncil/Basilisk-node/pull/174) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- fix: asset instance type ([@lumir-mrkva](https://github.com/lumir-mrkva))
- fix: checked runtime upgrade [#152](https://github.com/galacticcouncil/Basilisk-node/pull/152) ([@lumir-mrkva](https://github.com/lumir-mrkva) [@jak-pan](https://github.com/jak-pan))
- refactor: improve multi payment (builder review) [#170](https://github.com/galacticcouncil/Basilisk-node/pull/170) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- fix: builder review final changes [#172](https://github.com/galacticcouncil/Basilisk-node/pull/172) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- fix(exchange): fix matching algorithm [#155](https://github.com/galacticcouncil/Basilisk-node/pull/155) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@jak-pan](https://github.com/jak-pan))
- chore: Added an extra node to the rococo-local config [#165](https://github.com/galacticcouncil/Basilisk-node/pull/165) (matej.sima@stove-labs.com [@maht0rz](https://github.com/maht0rz) [@jak-pan](https://github.com/jak-pan))
- fix: base xcm weight [#163](https://github.com/galacticcouncil/Basilisk-node/pull/163) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- fix: audit changes [#162](https://github.com/galacticcouncil/Basilisk-node/pull/162) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- ci: add clippy [#156](https://github.com/galacticcouncil/Basilisk-node/pull/156) ([@Roznovjak](https://github.com/Roznovjak))
- fix: duster dependencies [#153](https://github.com/galacticcouncil/Basilisk-node/pull/153) ([@lumir-mrkva](https://github.com/lumir-mrkva) [@enthusiastmartin](https://github.com/enthusiastmartin))

#### Authors: 7

- [@green-jay](https://github.com/green-jay)
- [@lumir-mrkva](https://github.com/lumir-mrkva)
- Jakub Pánik ([@jak-pan](https://github.com/jak-pan))
- Martin Hloska ([@enthusiastmartin](https://github.com/enthusiastmartin))
- Matej Šima ([@maht0rz](https://github.com/maht0rz))
- Matej Sima (matej.sima@stove-labs.com)
- Richard Roznovjak ([@Roznovjak](https://github.com/Roznovjak))

---

# v4.0.0 (Tue Aug 31 2021)

#### 🚀 Enhancement

- chore: launch code review [#144](https://github.com/galacticcouncil/Basilisk-node/pull/144) ([@jak-pan](https://github.com/jak-pan) [@lumir-mrkva](https://github.com/lumir-mrkva) [@green-jay](https://github.com/green-jay) [@enthusiastmartin](https://github.com/enthusiastmartin) [@Roznovjak](https://github.com/Roznovjak))
- feat: existential deposits in asset registry [#146](https://github.com/galacticcouncil/Basilisk-node/pull/146) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@jak-pan](https://github.com/jak-pan))
- feat: NFT builder's program refactoring [#125](https://github.com/galacticcouncil/Basilisk-node/pull/125) ([@green-jay](https://github.com/green-jay) [@fakirAyoub](https://github.com/fakirAyoub) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: asset registry refactored [#124](https://github.com/galacticcouncil/Basilisk-node/pull/124) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@jak-pan](https://github.com/jak-pan) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: basilisk + karura launch config [#127](https://github.com/galacticcouncil/Basilisk-node/pull/127) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- feat: duster pallet [#104](https://github.com/galacticcouncil/Basilisk-node/pull/104) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: balance check as signed extension [#115](https://github.com/galacticcouncil/Basilisk-node/pull/115) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- feat: xcm reintroduced [#114](https://github.com/galacticcouncil/Basilisk-node/pull/114) ([@lumir-mrkva](https://github.com/lumir-mrkva) [@enthusiastmartin](https://github.com/enthusiastmartin))

#### 🐛 Bug Fix

- fix: fix json configs to work with latest polkadot-launch [#139](https://github.com/galacticcouncil/Basilisk-node/pull/139) ([@enthusiastmartin](https://github.com/enthusiastmartin) [@lumir-mrkva](https://github.com/lumir-mrkva))
- fix: parachain id for kusama [#103](https://github.com/galacticcouncil/Basilisk-node/pull/103) ([@martinfridrich](https://github.com/martinfridrich) [@lumir-mrkva](https://github.com/lumir-mrkva))
- fix: lbp [#142](https://github.com/galacticcouncil/Basilisk-node/pull/142) ([@Roznovjak](https://github.com/Roznovjak) [@lumir-mrkva](https://github.com/lumir-mrkva))
- fix: rebench nft weights [#143](https://github.com/galacticcouncil/Basilisk-node/pull/143) ([@green-jay](https://github.com/green-jay))
- fix: nft bench + remove emote [#138](https://github.com/galacticcouncil/Basilisk-node/pull/138) ([@green-jay](https://github.com/green-jay) [@lumir-mrkva](https://github.com/lumir-mrkva))
- chore: benchmarks and weights update [#140](https://github.com/galacticcouncil/Basilisk-node/pull/140) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- chore: updated to polkadot 0.9.9-1 [#133](https://github.com/galacticcouncil/Basilisk-node/pull/133) ([@lumir-mrkva](https://github.com/lumir-mrkva) [@enthusiastmartin](https://github.com/enthusiastmartin) [@Roznovjak](https://github.com/Roznovjak))
- ci: fix runner image [#134](https://github.com/galacticcouncil/Basilisk-node/pull/134) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- fix(vesting): fix benchmarking [#123](https://github.com/galacticcouncil/Basilisk-node/pull/123) ([@Roznovjak](https://github.com/Roznovjak))
- chore: Update to polkadot 0.9.8 [#105](https://github.com/galacticcouncil/Basilisk-node/pull/105) ([@lumir-mrkva](https://github.com/lumir-mrkva) [@enthusiastmartin](https://github.com/enthusiastmartin))
- fix: fix codecov yml [#121](https://github.com/galacticcouncil/Basilisk-node/pull/121) ([@enthusiastmartin](https://github.com/enthusiastmartin))
- ci: add codecov yml to adjust thresholds [#119](https://github.com/galacticcouncil/Basilisk-node/pull/119) ([@enthusiastmartin](https://github.com/enthusiastmartin))

#### Authors: 7

- [@green-jay](https://github.com/green-jay)
- [@lumir-mrkva](https://github.com/lumir-mrkva)
- Ayoub Fakir ([@fakirAyoub](https://github.com/fakirAyoub))
- Jakub Pánik ([@jak-pan](https://github.com/jak-pan))
- martin fridrich ([@martinfridrich](https://github.com/martinfridrich))
- Martin Hloska ([@enthusiastmartin](https://github.com/enthusiastmartin))
- Richard Roznovjak ([@Roznovjak](https://github.com/Roznovjak))

---

# v3.0.0 (Mon Jul 12 2021)

#### 🐛 Bug Fix

- chore!: use latest math package [#99](https://github.com/galacticcouncil/Basilisk-node/pull/99) ([@enthusiastmartin](https://github.com/enthusiastmartin))

#### Authors: 1

- Martin Hloska ([@enthusiastmartin](https://github.com/enthusiastmartin))

---

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
