#![cfg(test)]

use crate::kusama_test_net::*;
use basilisk_runtime::{
	AssetRegistry, Balances, ClassDeposit, InstanceDeposit, Marketplace, MinimumOfferAmount, Origin, RoyaltyBondAmount,
	Tokens, NFT, RELAY_CHAIN_ASSET_LOCATION,
};
use frame_support::{assert_noop, assert_ok};
use hydradx_traits::nft::CreateTypedClass;
use orml_traits::MultiCurrency;
use orml_traits::MultiReservableCurrency;
use pallet_nft::NftPermission;
use primitives::nft::ClassType;
use primitives::{AssetId, ClassId};
use test_case::test_case;
use xcm_emulator::TestExt;

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
			15
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

const ALLOW: bool = true;
const NOT_ALLOW: bool = false;

#[test_case(ClassType::Marketplace, ALLOW ; "marketplace class")]
#[test_case(ClassType::LiquidityMining, NOT_ALLOW ; "liquidity mining class")]
#[test_case(ClassType::Redeemable, NOT_ALLOW ; "redeemable class")]
#[test_case(ClassType::Auction, NOT_ALLOW ; "auction class")]
#[test_case(ClassType::HydraHeads, NOT_ALLOW ; "hydra heads class")]
fn test_nft_permission_for_class_creation(class_type: ClassType, is_allowed: bool) {
	assert_eq!(
		is_allowed,
		<basilisk_runtime::Runtime as pallet_nft::Config>::Permissions::can_create(&class_type)
	);
}

#[test_case(ClassType::Marketplace, ALLOW ; "marketplace class")]
#[test_case(ClassType::LiquidityMining, NOT_ALLOW ; "liquidity mining class")]
#[test_case(ClassType::Redeemable, NOT_ALLOW ; "redeemable class")]
#[test_case(ClassType::Auction, NOT_ALLOW ; "auction class")]
#[test_case(ClassType::HydraHeads, NOT_ALLOW ; "hydra heads class")]
fn test_nft_permission_for_minting(class_type: ClassType, is_allowed: bool) {
	assert_eq!(
		is_allowed,
		<basilisk_runtime::Runtime as pallet_nft::Config>::Permissions::can_mint(&class_type)
	);
}

#[test_case(ClassType::Marketplace, ALLOW ; "marketplace class")]
#[test_case(ClassType::LiquidityMining, ALLOW ; "liquidity mining class")]
#[test_case(ClassType::Redeemable, NOT_ALLOW ; "redeemable class")]
#[test_case(ClassType::Auction, NOT_ALLOW ; "auction class")]
#[test_case(ClassType::HydraHeads, NOT_ALLOW ; "hydra heads class")]
fn test_nft_permission_for_transfer(class_type: ClassType, is_allowed: bool) {
	assert_eq!(
		is_allowed,
		<basilisk_runtime::Runtime as pallet_nft::Config>::Permissions::can_transfer(&class_type)
	);
}

#[test_case(ClassType::Marketplace, ALLOW ; "marketplace class")]
#[test_case(ClassType::LiquidityMining, NOT_ALLOW ; "liquidity mining class")]
#[test_case(ClassType::Redeemable, NOT_ALLOW ; "redeemable class")]
#[test_case(ClassType::Auction, NOT_ALLOW ; "auction class")]
#[test_case(ClassType::HydraHeads, NOT_ALLOW ; "hydra heads class")]
fn test_nft_permission_for_burning(class_type: ClassType, is_allowed: bool) {
	assert_eq!(
		is_allowed,
		<basilisk_runtime::Runtime as pallet_nft::Config>::Permissions::can_burn(&class_type)
	);
}

#[test_case(ClassType::Marketplace, ALLOW ; "marketplace class")]
#[test_case(ClassType::LiquidityMining, NOT_ALLOW ; "liquidity mining class")]
#[test_case(ClassType::Redeemable, NOT_ALLOW ; "redeemable class")]
#[test_case(ClassType::Auction, NOT_ALLOW ; "auction class")]
#[test_case(ClassType::HydraHeads, NOT_ALLOW ; "hydra heads class")]
fn test_nft_permission_for_class_destroying(class_type: ClassType, is_allowed: bool) {
	assert_eq!(
		is_allowed,
		<basilisk_runtime::Runtime as pallet_nft::Config>::Permissions::can_destroy(&class_type)
	);
}

#[test_case(ClassType::Marketplace, ALLOW ; "marketplace class")]
#[test_case(ClassType::LiquidityMining, NOT_ALLOW ; "liquidity mining class")]
#[test_case(ClassType::Redeemable, NOT_ALLOW ; "redeemable class")]
#[test_case(ClassType::Auction, NOT_ALLOW ; "auction class")]
#[test_case(ClassType::HydraHeads, NOT_ALLOW ; "hydra heads class")]
fn test_nft_permission_for_deposit(class_type: ClassType, is_allowed: bool) {
	assert_eq!(
		is_allowed,
		<basilisk_runtime::Runtime as pallet_nft::Config>::Permissions::has_deposit(&class_type)
	);
}

#[test_case(ClassType::Marketplace; "marketplace class")]
#[test_case(ClassType::LiquidityMining; "liquidity mining class")]
#[test_case(ClassType::Redeemable; "redeemable class")]
#[test_case(ClassType::Auction; "auction class")]
#[test_case(ClassType::HydraHeads; "hydra heads class")]
fn create_class_should_be_without_deposit_when_created_using_trait(class_type: ClassType) {
	// Arrange
	TestNet::reset();
	Basilisk::execute_with(|| {
		let initial_balance = Balances::free_balance(&AccountId::from(ALICE));

		// Act & Assert
		assert_ok!(
			<NFT as CreateTypedClass<AccountId, ClassId, ClassType>>::create_typed_class(
				AccountId::from(ALICE),
				0,
				class_type,
			)
		);

		assert_eq!(initial_balance, Balances::free_balance(&AccountId::from(ALICE)));
	});
}
