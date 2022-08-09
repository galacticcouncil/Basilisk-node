#![cfg(test)]

use crate::kusama_test_net::*;
use basilisk_runtime::{
	AssetRegistry, ClassDeposit, InstanceDeposit, Marketplace, MinimumOfferAmount, Origin, RoyaltyBondAmount, Tokens,
	NFT, RELAY_CHAIN_ASSET_LOCATION,
};
use frame_support::{assert_noop, assert_ok};
use orml_traits::MultiCurrency;
use orml_traits::MultiReservableCurrency;
use primitives::nft::ClassType;
use primitives::{AssetId, ClassId};
use xcm_emulator::TestExt;
use sp_arithmetic::Percent;

const KSM: AssetId = 1;

const ALICE_COLLECTION: ClassId = 13370000;

fn init() {
	TestNet::reset();
	Basilisk::execute_with(|| {
		assert_ok!(AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			KSM,
			basilisk_runtime::AssetLocation(RELAY_CHAIN_ASSET_LOCATION.0)
		));
	});
}

fn arrange_nft() {
	Basilisk::execute_with(|| {
		assert_ok!(NFT::create_class(
			Origin::signed(ALICE.into()),
			ALICE_COLLECTION,
			ClassType::Marketplace,
			b"ipfs://QmZn9GFNrNyaTXNdCLWEPtjYHGG9yajgw9JzxpMoDZ2Ziq"
				.to_vec()
				.try_into()
				.unwrap(),
		));
		assert_ok!(NFT::mint(
			Origin::signed(ALICE.into()),
			ALICE_COLLECTION,
			0,
			b"ipfs://QmQu2jUmtFNPd86tEHFs6hmAArKYyjEC3xuwVWpFGjcMgm"
				.to_vec()
				.try_into()
				.unwrap(),
		));
	});
}

#[test]
fn ksm_should_have_relay_chain_asset_location_on_init() {
	init();
	Basilisk::execute_with(|| {
		assert_eq!(AssetRegistry::asset_to_location(KSM), Some(RELAY_CHAIN_ASSET_LOCATION));
	});
}

#[test]
fn bob_should_have_ksm_on_init() {
	init();
	Basilisk::execute_with(|| {
		assert_eq!(Tokens::free_balance(KSM, &AccountId::from(BOB)), 1000 * UNITS);
	});
}

#[test]
fn nft_pallet_should_reserve_ksm_when_nft_is_arranged() {
	init();
	arrange_nft();
	Basilisk::execute_with(|| {
		assert_eq!(
			Tokens::reserved_balance(KSM, &AccountId::from(ALICE)),
			ClassDeposit::get() + InstanceDeposit::get()
		);
	});
}

#[test]
fn marketplace_should_reserve_ksm_when_royalties_are_added() {
	init();
	arrange_nft();
	Basilisk::execute_with(|| {
		assert_ok!(Marketplace::add_royalty(
			Origin::signed(ALICE.into()),
			ALICE_COLLECTION,
			0,
			AccountId::from(ALICE),
			Percent::from_percent(15)
		));
		assert_eq!(
			Tokens::reserved_balance(KSM, &AccountId::from(ALICE)),
			ClassDeposit::get() + InstanceDeposit::get() + RoyaltyBondAmount::get()
		);
	});
}

#[test]
fn make_offer_should_reserve_ksm_when_created() {
	init();
	arrange_nft();
	Basilisk::execute_with(|| {
		assert_ok!(Marketplace::make_offer(
			Origin::signed(BOB.into()),
			ALICE_COLLECTION,
			0,
			MinimumOfferAmount::get(),
			10
		));
		assert_eq!(
			Tokens::reserved_balance(KSM, &AccountId::from(BOB)),
			MinimumOfferAmount::get()
		);
	});
}

#[test]
#[ignore] // is not case when the minting is free
fn create_class_should_fail_when_relay_chain_location_not_registered() {
	TestNet::reset();
	Basilisk::execute_with(|| {
		assert_noop!(
			NFT::create_class(
				Origin::signed(ALICE.into()),
				ALICE_COLLECTION,
				ClassType::Marketplace,
				b"ipfs://QmZn9GFNrNyaTXNdCLWEPtjYHGG9yajgw9JzxpMoDZ2Ziq"
					.to_vec()
					.try_into()
					.unwrap(),
			),
			orml_tokens::Error::<basilisk_runtime::Runtime>::BalanceTooLow
		);
	});
}
