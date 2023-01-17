use super::*;
use crate::{
	AccountId, AssetRegistry, Marketplace, RelayChainAssetId, Runtime, Uniques, NFT, RELAY_CHAIN_ASSET_LOCATION,
};
use frame_benchmarking::{account, vec};
use frame_support::{
	sp_runtime::{traits::StaticLookup, SaturatedConversion},
	traits::Get,
	BoundedVec,
};
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use pallet_nft::BoundedVecOfUnq;
use primitives::{constants::currency::UNITS, CollectionId, ItemId};
use registry_traits::registry::Registry;
use sp_std::convert::TryInto;

const ENDOWMENT: u32 = 1_000_000;
const COLLECTION_ID_0: u32 = 1_000_000;
const ITEM_ID_0: u32 = 0;

pub fn create_account(name: &'static str) -> AccountId {
	let acc = account(name, 0, 0u32);
	update_balance(RelayChainAssetId::get(), &acc, ENDOWMENT as u128 * UNITS);
	acc
}

fn create_collection_and_mint(
	collection_id: CollectionId,
	item_id: ItemId,
) -> (
	AccountId,
	AccountId,
	<<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source,
	BoundedVecOfUnq<Runtime>,
) {
	let name = vec![1; <Runtime as pallet_asset_registry::Config>::StringLimit::get() as usize];
	assert_ok!(AssetRegistry::register(
		RawOrigin::Root.into(),
		name.clone(),
		pallet_asset_registry::AssetType::Token,
		1_000u32.into()
	));
	let asset_id = AssetRegistry::retrieve_asset(&name).unwrap();
	assert_ok!(AssetRegistry::set_location(
		RawOrigin::Root.into(),
		asset_id,
		RELAY_CHAIN_ASSET_LOCATION
	));

	let caller = create_account("caller");
	let caller2 = create_account("caller2");
	let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
	let metadata: BoundedVec<_, _> = vec![0; <Runtime as pallet_uniques::Config>::StringLimit::get() as usize]
		.try_into()
		.unwrap();

	assert!(NFT::create_collection(
		RawOrigin::Signed(caller.clone()).into(),
		collection_id,
		Default::default(),
		metadata.clone()
	)
	.is_ok());

	assert!(NFT::mint(
		RawOrigin::Signed(caller.clone()).into(),
		collection_id,
		item_id,
		metadata.clone()
	)
	.is_ok());
	(caller, caller2, caller_lookup, metadata)
}

runtime_benchmarks! {
	{ Runtime, pallet_marketplace}

	buy {
		let (caller, caller2, caller_lookup, metadata) = create_collection_and_mint(COLLECTION_ID_0.into(), ITEM_ID_0.into());
		Marketplace::set_price(RawOrigin::Signed(caller).into(), COLLECTION_ID_0.into(), ITEM_ID_0.into(), Some(1u32.into()))?;
	}: _(RawOrigin::Signed(caller2.clone()), COLLECTION_ID_0.into(), ITEM_ID_0.into())
	verify {
		assert_eq!(Uniques::owner(CollectionId::from(COLLECTION_ID_0), ItemId::from(ITEM_ID_0)), Some(caller2))
	}

	set_price {
		let (caller, caller2, caller_lookup, metadata) = create_collection_and_mint(COLLECTION_ID_0.into(), ITEM_ID_0.into());
	}: _(RawOrigin::Signed(caller.clone()), COLLECTION_ID_0.into(), ITEM_ID_0.into(), Some(u32::max_value().into()))
	verify {
		assert_eq!(Marketplace::prices(CollectionId::from(COLLECTION_ID_0), ItemId::from(ITEM_ID_0)), Some(u32::max_value().into()))
	}

	make_offer {
		let (caller, caller2, caller_lookup, metadata) = create_collection_and_mint(COLLECTION_ID_0.into(), ITEM_ID_0.into());
	}: _(RawOrigin::Signed(caller.clone()), COLLECTION_ID_0.into(), ITEM_ID_0.into(), (100_000 * UNITS).saturated_into(), 666u32)
	verify {
		assert!(
			Marketplace::offers((CollectionId::from(COLLECTION_ID_0), ItemId::from(ITEM_ID_0)), caller).is_some()
		)
	}

	withdraw_offer {
		let (caller, caller2, caller_lookup, metadata) = create_collection_and_mint(COLLECTION_ID_0.into(), ITEM_ID_0.into());
		Marketplace::make_offer(RawOrigin::Signed(caller2.clone()).into(), COLLECTION_ID_0.into(), ITEM_ID_0.into(), (100_000 * UNITS).saturated_into(), 666u32)?;
	}: _(RawOrigin::Signed(caller2.clone()), COLLECTION_ID_0.into(), ITEM_ID_0.into(), caller2.clone())
	verify {
		assert_eq!(
			Marketplace::offers((CollectionId::from(COLLECTION_ID_0), ItemId::from(ITEM_ID_0)), caller2),
			None
		)
	}

	accept_offer {
		let (caller, caller2, caller_lookup, metadata) = create_collection_and_mint(COLLECTION_ID_0.into(), ITEM_ID_0.into());
		Marketplace::make_offer(RawOrigin::Signed(caller2.clone()).into(), COLLECTION_ID_0.into(), ITEM_ID_0.into(), (100_000 * UNITS).saturated_into(), 666u32)?;
	}: _(RawOrigin::Signed(caller), COLLECTION_ID_0.into(), ITEM_ID_0.into(), caller2.clone())
	verify {
		assert_eq!(
			Marketplace::offers((CollectionId::from(COLLECTION_ID_0), ItemId::from(ITEM_ID_0)), caller2),
			None
		)
	}

	add_royalty {
		let (caller, caller2, caller_lookup, metadata) = create_collection_and_mint(COLLECTION_ID_0.into(), ITEM_ID_0.into());
	}: _(RawOrigin::Signed(caller), COLLECTION_ID_0.into(), ITEM_ID_0.into(), caller2, 2_500u16)
	verify {
		assert!(
			Marketplace::marketplace_items(CollectionId::from(COLLECTION_ID_0), ItemId::from(ITEM_ID_0)).is_some()
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use orml_benchmarking::impl_benchmark_test_suite;

	fn new_test_ext() -> sp_io::TestExternalities {
		frame_system::GenesisConfig::default()
			.build_storage::<crate::Runtime>()
			.unwrap()
			.into()
	}

	impl_benchmark_test_suite!(new_test_ext(),);
}
