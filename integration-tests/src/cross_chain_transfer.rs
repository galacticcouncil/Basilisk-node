#![cfg(test)]
use crate::kusama_test_net::*;

use frame_support::assert_ok;
use polkadot_xcm::v0::{
	Junction::{self, GeneralKey, Parachain, Parent},
	MultiAsset::*,
	MultiLocation::*,
	NetworkId,
};

use cumulus_primitives_core::ParaId;
use orml_traits::currency::MultiCurrency;
use sp_runtime::traits::AccountIdConversion;
use xcm_emulator::TestExt;

#[test]
fn transfer_from_relay_chain() {
	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(X1(Parent,))
		));
	});
	Kusama::execute_with(|| {
		assert_ok!(kusama_runtime::XcmPallet::reserve_transfer_assets(
			kusama_runtime::Origin::signed(ALICE.into()),
			X1(Parachain(2000)),
			X1(Junction::AccountId32 {
				id: BOB,
				network: NetworkId::Any
			}),
			vec![ConcreteFungible {
				id: Null,
				amount: 3 * BSX
			}],
			1_600_000_000
		));

		assert_eq!(
			kusama_runtime::Balances::free_balance(&ParaId::from(2000).into_account()),
			13 * BSX
		);
	});

	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			3 * BSX
		);
	});
}

#[test]
fn transfer_to_relay_chain() {
	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(X1(Parent,))
		));

		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::Origin::signed(ALICE.into()),
			1,
			3 * BSX,
			X2(
				Parent,
				Junction::AccountId32 {
					id: BOB,
					network: NetworkId::Any,
				}
			),
			3_600_000_000
		));
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(ALICE)),
			200 * BSX - 3 * BSX
		);
	});

	Kusama::execute_with(|| {
		assert_eq!(
			kusama_runtime::Balances::free_balance(&AccountId::from(BOB)),
			2999904000006 // 3 * BSX - fee
		);
	});
}

#[test]
fn transfer_from_hydra() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(X3(Parent, Parachain(3000), GeneralKey(vec![0, 0, 0, 0])))
		));
	});

	Hydra::execute_with(|| {
		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::Origin::signed(ALICE.into()),
			0,
			3 * BSX,
			X3(
				Parent,
				Junction::Parachain(2000),
				Junction::AccountId32 {
					id: BOB,
					network: NetworkId::Any,
				}
			),
			399_600_000_000
		));
		assert_eq!(
			basilisk_runtime::Balances::free_balance(&AccountId::from(ALICE)),
			200 * BSX - 3 * BSX
		);
	});

	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			3 * BSX
		);
	});
}
