#![cfg(test)]

use crate::kusama_test_net::*;
use basilisk_runtime::NFT;
use frame_support::assert_ok;
use frame_support::traits::tokens::nonfungibles::*;
use frame_support::traits::ReservableCurrency;
use hydradx_traits::nft::CreateTypedClass;
use pallet_nft::NftPermission;
use pallet_nft::ClassType;
use primitives::{ClassId, InstanceId};
use test_case::test_case;
use xcm_emulator::TestExt;

const ALLOW: bool = true;
const NOT_ALLOW: bool = false;
const RESERVED_CLASS_ID: ClassId = 0;
const RESTRICTED_CLASS_TYPE: ClassType = ClassType::HydraHeads;

fn create_nft_class(account_id: AccountId, class_id: ClassId, class_type: ClassType) {
	Basilisk::execute_with(|| {
		assert_ok!(
			<NFT as CreateTypedClass<AccountId, ClassId, ClassType>>::create_typed_class(
				account_id, class_id, class_type,
			)
		);
	});
}

fn mint_nft(account_id: AccountId, class_id: ClassId, instance_id: InstanceId) {
	Basilisk::execute_with(|| {
		assert_ok!(<NFT as Mutate<AccountId>>::mint_into(
			&class_id,
			&instance_id,
			&account_id,
		));
	});
}

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

// Nonfungibles traits

#[test_case(ClassType::Marketplace; "marketplace class")]
#[test_case(ClassType::LiquidityMining; "liquidity mining class")]
#[test_case(ClassType::Redeemable; "redeemable class")]
#[test_case(ClassType::Auction; "auction class")]
#[test_case(ClassType::HydraHeads; "hydra heads class")]
fn deposit_for_create_typed_class_should_be_zero(class_type: ClassType) {
	// Arrange
	TestNet::reset();
	Basilisk::execute_with(|| {
		// Act & Assert
		assert_ok!(
			<NFT as CreateTypedClass<AccountId, ClassId, ClassType>>::create_typed_class(
				AccountId::from(ALICE),
				RESERVED_CLASS_ID,
				class_type,
			)
		);

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
		assert_ok!(<NFT as Create<AccountId>>::create_class(
			&RESERVED_CLASS_ID,
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
fn create_class_should_ignore_reserved_ids() {
	// Arrange
	TestNet::reset();
	Basilisk::execute_with(|| {
		// Act & Assert
		assert_ok!(<NFT as Create<AccountId>>::create_class(
			&RESERVED_CLASS_ID,
			&AccountId::from(ALICE),
			&AccountId::from(ALICE),
		));
	});
}

#[test]
fn destroy_class_should_ignore_permissions() {
	// Arrange
	TestNet::reset();
	create_nft_class(AccountId::from(ALICE), RESERVED_CLASS_ID, RESTRICTED_CLASS_TYPE);
	Basilisk::execute_with(|| {
		// Act & Assert
		assert_ok!(<NFT as Destroy<AccountId>>::destroy(
			RESERVED_CLASS_ID,
			<NFT as Destroy<AccountId>>::get_destroy_witness(&RESERVED_CLASS_ID).unwrap(),
			None,
		));
	});
}

#[test]
fn mint_into_should_ignore_permissions() {
	// Arrange
	let instance_id = 0;
	TestNet::reset();
	create_nft_class(AccountId::from(ALICE), RESERVED_CLASS_ID, RESTRICTED_CLASS_TYPE);
	Basilisk::execute_with(|| {
		// Act & Assert
		assert_ok!(<NFT as Mutate<AccountId>>::mint_into(
			&RESERVED_CLASS_ID,
			&instance_id,
			&AccountId::from(ALICE)
		));
	});
}

#[test]
fn burn_from_should_ignore_permissions() {
	// Arrange
	let instance_id = 0;
	TestNet::reset();
	create_nft_class(AccountId::from(ALICE), RESERVED_CLASS_ID, RESTRICTED_CLASS_TYPE);
	mint_nft(AccountId::from(ALICE), RESERVED_CLASS_ID, instance_id);
	Basilisk::execute_with(|| {
		// Act & Assert
		assert_ok!(<NFT as Mutate<AccountId>>::burn_from(&RESERVED_CLASS_ID, &instance_id,));
	});
}

#[test]
fn transfer_should_ignore_permissions() {
	// Arrange
	let instance_id = 0;
	TestNet::reset();
	create_nft_class(AccountId::from(ALICE), RESERVED_CLASS_ID, RESTRICTED_CLASS_TYPE);
	mint_nft(AccountId::from(ALICE), RESERVED_CLASS_ID, instance_id);
	Basilisk::execute_with(|| {
		// Act & Assert
		assert_ok!(<NFT as Transfer<AccountId>>::transfer(
			&RESERVED_CLASS_ID,
			&instance_id,
			&AccountId::from(ALICE),
		));
	});
}

#[test_case(ClassType::Marketplace; "marketplace class")]
#[test_case(ClassType::LiquidityMining; "liquidity mining class")]
#[test_case(ClassType::Redeemable; "redeemable class")]
#[test_case(ClassType::Auction; "auction class")]
#[test_case(ClassType::HydraHeads; "hydra heads class")]
fn create_typed_class_should_ignore_permissions_and_reserved_ids(class_type: ClassType) {
	// Arrange
	TestNet::reset();
	Basilisk::execute_with(|| {
		// Act & Assert
		assert_ok!(
			<NFT as CreateTypedClass<AccountId, ClassId, ClassType>>::create_typed_class(
				AccountId::from(ALICE),
				RESERVED_CLASS_ID,
				class_type,
			)
		);
	});
}
