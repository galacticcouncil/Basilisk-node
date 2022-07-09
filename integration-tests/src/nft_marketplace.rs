#![cfg(test)]

use crate::kusama_test_net::*;
use basilisk_runtime::{
	AssetRegistry, Balances, ClassDeposit, InstanceDeposit, Marketplace, MinimumOfferAmount, Origin, RoyaltyBondAmount,
	Tokens, NFT, RELAY_CHAIN_ASSET_LOCATION,
};
use frame_support::assert_ok;
use orml_traits::MultiCurrency;
use orml_traits::MultiReservableCurrency;
use primitives::nft::ClassType;
use primitives::{AssetId, ClassId};
use xcm_emulator::TestExt;

const KSM: AssetId = 1;

const ALICE_COLLECTION: ClassId = 13370000;

fn init() {
	TestNet::reset();
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
		assert_eq!(Tokens::free_balance(1, &AccountId::from(BOB)), 200 * UNITS);
	});
}

#[test]
fn nft_pallet_should_reserve_bsx_when_nft_is_arranged() {
	init();
	arrange_nft();
	Basilisk::execute_with(|| {
		assert_eq!(
			Balances::reserved_balance(&AccountId::from(ALICE)),
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
			15
		));
		assert_eq!(
			Tokens::reserved_balance(KSM, &AccountId::from(ALICE)),
			RoyaltyBondAmount::get()
		);
	});
}

#[test]
fn marketplace_should_reserve_ksm_when_offer_is_created() {
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
