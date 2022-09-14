#![cfg(test)]

use crate::kusama_test_net::*;
use basilisk_runtime::NFT;
use frame_support::assert_ok;
use frame_support::traits::tokens::nonfungibles::*;
use frame_support::traits::ReservableCurrency;
use hydradx_traits::nft::CreateTypedCollection;
use pallet_nft::CollectionType;
use pallet_nft::NftPermission;
use primitives::{CollectionId, ItemId};
use test_case::test_case;
use xcm_emulator::TestExt;

const ALLOW: bool = true;
const NOT_ALLOW: bool = false;
const RESERVED_COLLECTION_ID: CollectionId = 0;
const RESTRICTED_COLLECTION_TYPE: CollectionType = CollectionType::HydraHeads;

fn create_nft_collection(account_id: AccountId, collection_id: CollectionId, collection_type: CollectionType) {
	Basilisk::execute_with(|| {
		assert_ok!(<NFT as CreateTypedCollection<
			AccountId,
			CollectionId,
			CollectionType,
		>>::create_typed_collection(account_id, collection_id, collection_type,));
	});
}

fn mint_nft(account_id: AccountId, collection_id: CollectionId, item_id: ItemId) {
	Basilisk::execute_with(|| {
		assert_ok!(<NFT as Mutate<AccountId>>::mint_into(
			&collection_id,
			&item_id,
			&account_id,
		));
	});
}

#[test_case(CollectionType::Marketplace, ALLOW ; "marketplace collection")]
#[test_case(CollectionType::LiquidityMining, NOT_ALLOW ; "liquidity mining collection")]
#[test_case(CollectionType::Redeemable, NOT_ALLOW ; "redeemable collection")]
#[test_case(CollectionType::Auction, NOT_ALLOW ; "auction collection")]
#[test_case(CollectionType::HydraHeads, NOT_ALLOW ; "hydra heads collection")]
fn test_nft_permission_for_collection_creation(collection_type: CollectionType, is_allowed: bool) {
	assert_eq!(
		is_allowed,
		<basilisk_runtime::Runtime as pallet_nft::Config>::Permissions::can_create(&collection_type)
	);
}

#[test_case(CollectionType::Marketplace, ALLOW ; "marketplace collection")]
#[test_case(CollectionType::LiquidityMining, NOT_ALLOW ; "liquidity mining collection")]
#[test_case(CollectionType::Redeemable, NOT_ALLOW ; "redeemable collection")]
#[test_case(CollectionType::Auction, NOT_ALLOW ; "auction collection")]
#[test_case(CollectionType::HydraHeads, NOT_ALLOW ; "hydra heads collection")]
fn test_nft_permission_for_minting(collection_type: CollectionType, is_allowed: bool) {
	assert_eq!(
		is_allowed,
		<basilisk_runtime::Runtime as pallet_nft::Config>::Permissions::can_mint(&collection_type)
	);
}

#[test_case(CollectionType::Marketplace, ALLOW ; "marketplace collection")]
#[test_case(CollectionType::LiquidityMining, ALLOW ; "liquidity mining collection")]
#[test_case(CollectionType::Redeemable, NOT_ALLOW ; "redeemable collection")]
#[test_case(CollectionType::Auction, NOT_ALLOW ; "auction collection")]
#[test_case(CollectionType::HydraHeads, NOT_ALLOW ; "hydra heads collection")]
fn test_nft_permission_for_transfer(collection_type: CollectionType, is_allowed: bool) {
	assert_eq!(
		is_allowed,
		<basilisk_runtime::Runtime as pallet_nft::Config>::Permissions::can_transfer(&collection_type)
	);
}

#[test_case(CollectionType::Marketplace, ALLOW ; "marketplace collection")]
#[test_case(CollectionType::LiquidityMining, NOT_ALLOW ; "liquidity mining collection")]
#[test_case(CollectionType::Redeemable, NOT_ALLOW ; "redeemable collection")]
#[test_case(CollectionType::Auction, NOT_ALLOW ; "auction collection")]
#[test_case(CollectionType::HydraHeads, NOT_ALLOW ; "hydra heads collection")]
fn test_nft_permission_for_burning(collection_type: CollectionType, is_allowed: bool) {
	assert_eq!(
		is_allowed,
		<basilisk_runtime::Runtime as pallet_nft::Config>::Permissions::can_burn(&collection_type)
	);
}

#[test_case(CollectionType::Marketplace, ALLOW ; "marketplace collection")]
#[test_case(CollectionType::LiquidityMining, NOT_ALLOW ; "liquidity mining collection")]
#[test_case(CollectionType::Redeemable, NOT_ALLOW ; "redeemable collection")]
#[test_case(CollectionType::Auction, NOT_ALLOW ; "auction collection")]
#[test_case(CollectionType::HydraHeads, NOT_ALLOW ; "hydra heads collection")]
fn test_nft_permission_for_collection_destroying(collection_type: CollectionType, is_allowed: bool) {
	assert_eq!(
		is_allowed,
		<basilisk_runtime::Runtime as pallet_nft::Config>::Permissions::can_destroy(&collection_type)
	);
}

#[test_case(CollectionType::Marketplace, ALLOW ; "marketplace collection")]
#[test_case(CollectionType::LiquidityMining, NOT_ALLOW ; "liquidity mining collection")]
#[test_case(CollectionType::Redeemable, NOT_ALLOW ; "redeemable collection")]
#[test_case(CollectionType::Auction, NOT_ALLOW ; "auction collection")]
#[test_case(CollectionType::HydraHeads, NOT_ALLOW ; "hydra heads collection")]
fn test_nft_permission_for_deposit(collection_type: CollectionType, is_allowed: bool) {
	assert_eq!(
		is_allowed,
		<basilisk_runtime::Runtime as pallet_nft::Config>::Permissions::has_deposit(&collection_type)
	);
}

// Nonfungibles traits

#[test_case(CollectionType::Marketplace; "marketplace collection")]
#[test_case(CollectionType::LiquidityMining; "liquidity mining collection")]
#[test_case(CollectionType::Redeemable; "redeemable collection")]
#[test_case(CollectionType::Auction; "auction collection")]
#[test_case(CollectionType::HydraHeads; "hydra heads collection")]
fn deposit_for_create_typed_collection_should_be_zero(collection_type: CollectionType) {
	// Arrange
	TestNet::reset();
	Basilisk::execute_with(|| {
		// Act & Assert
		assert_ok!(<NFT as CreateTypedCollection<
			AccountId,
			CollectionId,
			CollectionType,
		>>::create_typed_collection(
			AccountId::from(ALICE), RESERVED_COLLECTION_ID, collection_type,
		));

		assert_eq!(
			<basilisk_runtime::Runtime as pallet_uniques::Config>::Currency::reserved_balance(&AccountId::from(ALICE)),
			0
		);
	});
}

#[test]
fn deposit_for_create_collection_should_be_zero() {
	// Arrange
	TestNet::reset();
	Basilisk::execute_with(|| {
		// Act & Assert
		assert_ok!(<NFT as Create<AccountId>>::create_collection(
			&RESERVED_COLLECTION_ID,
			&AccountId::from(ALICE),
			&AccountId::from(ALICE),
		));

		assert_eq!(
			<basilisk_runtime::Runtime as pallet_uniques::Config>::Currency::reserved_balance(&AccountId::from(ALICE)),
			0
		);
	});
}

#[test]
fn create_collection_should_ignore_reserved_ids() {
	// Arrange
	TestNet::reset();
	Basilisk::execute_with(|| {
		// Act & Assert
		assert_ok!(<NFT as Create<AccountId>>::create_collection(
			&RESERVED_COLLECTION_ID,
			&AccountId::from(ALICE),
			&AccountId::from(ALICE),
		));
	});
}

#[test]
fn destroy_collection_should_ignore_permissions() {
	// Arrange
	TestNet::reset();
	create_nft_collection(AccountId::from(ALICE), RESERVED_COLLECTION_ID, RESTRICTED_COLLECTION_TYPE);
	Basilisk::execute_with(|| {
		// Act & Assert
		assert_ok!(<NFT as Destroy<AccountId>>::destroy(
			RESERVED_COLLECTION_ID,
			<NFT as Destroy<AccountId>>::get_destroy_witness(&RESERVED_COLLECTION_ID).unwrap(),
			None,
		));
	});
}

#[test]
fn mint_into_should_ignore_permissions() {
	// Arrange
	let item_id = 0;
	TestNet::reset();
	create_nft_collection(AccountId::from(ALICE), RESERVED_COLLECTION_ID, RESTRICTED_COLLECTION_TYPE);
	Basilisk::execute_with(|| {
		// Act & Assert
		assert_ok!(<NFT as Mutate<AccountId>>::mint_into(
			&RESERVED_COLLECTION_ID,
			&item_id,
			&AccountId::from(ALICE)
		));
	});
}

#[test]
fn burn_from_should_ignore_permissions() {
	// Arrange
	let item_id = 0;
	TestNet::reset();
	create_nft_collection(AccountId::from(ALICE), RESERVED_COLLECTION_ID, RESTRICTED_COLLECTION_TYPE);
	mint_nft(AccountId::from(ALICE), RESERVED_COLLECTION_ID, item_id);
	Basilisk::execute_with(|| {
		// Act & Assert
		assert_ok!(<NFT as Mutate<AccountId>>::burn(&RESERVED_COLLECTION_ID, &item_id, None));
	});
}

#[test]
fn transfer_should_ignore_permissions() {
	// Arrange
	let item_id = 0;
	TestNet::reset();
	create_nft_collection(AccountId::from(ALICE), RESERVED_COLLECTION_ID, RESTRICTED_COLLECTION_TYPE);
	mint_nft(AccountId::from(ALICE), RESERVED_COLLECTION_ID, item_id);
	Basilisk::execute_with(|| {
		// Act & Assert
		assert_ok!(<NFT as Transfer<AccountId>>::transfer(
			&RESERVED_COLLECTION_ID,
			&item_id,
			&AccountId::from(ALICE),
		));
	});
}

#[test_case(CollectionType::Marketplace; "marketplace collection")]
#[test_case(CollectionType::LiquidityMining; "liquidity mining collection")]
#[test_case(CollectionType::Redeemable; "redeemable collection")]
#[test_case(CollectionType::Auction; "auction collection")]
#[test_case(CollectionType::HydraHeads; "hydra heads collection")]
fn create_typed_collection_should_ignore_permissions_and_reserved_ids(collection_type: CollectionType) {
	// Arrange
	TestNet::reset();
	Basilisk::execute_with(|| {
		// Act & Assert
		assert_ok!(<NFT as CreateTypedCollection<
			AccountId,
			CollectionId,
			CollectionType,
		>>::create_typed_collection(
			AccountId::from(ALICE), RESERVED_COLLECTION_ID, collection_type,
		));
	});
}
