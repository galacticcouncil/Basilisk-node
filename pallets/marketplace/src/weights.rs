#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_marketplace.
pub trait WeightInfo {
	fn set_price() -> Weight;
	fn buy() -> Weight;
	fn make_offer() -> Weight;
	fn withdraw_offer() -> Weight;
	fn accept_offer() -> Weight;
	fn add_royalty() -> Weight;
}

pub struct BasiliskWeight<T>(PhantomData<T>);

/// Weights for `pallet_marketplace`.
impl<T: frame_system::Config> WeightInfo for BasiliskWeight<T> {
	/// Storage: `Uniques::Asset` (r:1 w:1)
	/// Proof: `Uniques::Asset` (`max_values`: None, `max_size`: Some(146), added: 2621, mode: `MaxEncodedLen`)
	/// Storage: `Marketplace::Prices` (r:1 w:1)
	/// Proof: `Marketplace::Prices` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// Storage: `Marketplace::MarketplaceItems` (r:1 w:0)
	/// Proof: `Marketplace::MarketplaceItems` (`max_values`: None, `max_size`: Some(98), added: 2573, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::NextAssetId` (r:1 w:0)
	/// Proof: `AssetRegistry::NextAssetId` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::LocationAssets` (r:1 w:0)
	/// Proof: `AssetRegistry::LocationAssets` (`max_values`: None, `max_size`: Some(622), added: 3097, mode: `MaxEncodedLen`)
	/// Storage: `Tokens::Accounts` (r:2 w:2)
	/// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(108), added: 2583, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::Assets` (r:1 w:0)
	/// Proof: `AssetRegistry::Assets` (`max_values`: None, `max_size`: Some(87), added: 2562, mode: `MaxEncodedLen`)
	/// Storage: `NFT::Collections` (r:1 w:0)
	/// Proof: `NFT::Collections` (`max_values`: None, `max_size`: Some(99), added: 2574, mode: `MaxEncodedLen`)
	/// Storage: `Uniques::Class` (r:1 w:0)
	/// Proof: `Uniques::Class` (`max_values`: None, `max_size`: Some(190), added: 2665, mode: `MaxEncodedLen`)
	/// Storage: `Uniques::Account` (r:0 w:2)
	/// Proof: `Uniques::Account` (`max_values`: None, `max_size`: Some(112), added: 2587, mode: `MaxEncodedLen`)
	/// Storage: `Uniques::ItemPriceOf` (r:0 w:1)
	/// Proof: `Uniques::ItemPriceOf` (`max_values`: None, `max_size`: Some(113), added: 2588, mode: `MaxEncodedLen`)
	fn buy() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2792`
		//  Estimated: `6156`
		// Minimum execution time: 133_358_000 picoseconds.
		Weight::from_parts(134_664_000, 6156)
			.saturating_add(RocksDbWeight::get().reads(10_u64))
			.saturating_add(RocksDbWeight::get().writes(7_u64))
	}
	/// Storage: `Uniques::Asset` (r:1 w:0)
	/// Proof: `Uniques::Asset` (`max_values`: None, `max_size`: Some(146), added: 2621, mode: `MaxEncodedLen`)
	/// Storage: `Marketplace::Prices` (r:1 w:1)
	/// Proof: `Marketplace::Prices` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	fn set_price() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1360`
		//  Estimated: `3611`
		// Minimum execution time: 39_494_000 picoseconds.
		Weight::from_parts(39_928_000, 3611)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `Marketplace::Offers` (r:1 w:1)
	/// Proof: `Marketplace::Offers` (`max_values`: None, `max_size`: Some(148), added: 2623, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::NextAssetId` (r:1 w:0)
	/// Proof: `AssetRegistry::NextAssetId` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::LocationAssets` (r:1 w:0)
	/// Proof: `AssetRegistry::LocationAssets` (`max_values`: None, `max_size`: Some(622), added: 3097, mode: `MaxEncodedLen`)
	/// Storage: `Tokens::Accounts` (r:1 w:1)
	/// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(108), added: 2583, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::Assets` (r:1 w:0)
	/// Proof: `AssetRegistry::Assets` (`max_values`: None, `max_size`: Some(87), added: 2562, mode: `MaxEncodedLen`)
	fn make_offer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1731`
		//  Estimated: `4087`
		// Minimum execution time: 68_389_000 picoseconds.
		Weight::from_parts(69_155_000, 4087)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `Marketplace::Offers` (r:1 w:1)
	/// Proof: `Marketplace::Offers` (`max_values`: None, `max_size`: Some(148), added: 2623, mode: `MaxEncodedLen`)
	/// Storage: `Uniques::Asset` (r:1 w:0)
	/// Proof: `Uniques::Asset` (`max_values`: None, `max_size`: Some(146), added: 2621, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::NextAssetId` (r:1 w:0)
	/// Proof: `AssetRegistry::NextAssetId` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::LocationAssets` (r:1 w:0)
	/// Proof: `AssetRegistry::LocationAssets` (`max_values`: None, `max_size`: Some(622), added: 3097, mode: `MaxEncodedLen`)
	/// Storage: `Tokens::Accounts` (r:1 w:1)
	/// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(108), added: 2583, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::Assets` (r:1 w:0)
	/// Proof: `AssetRegistry::Assets` (`max_values`: None, `max_size`: Some(87), added: 2562, mode: `MaxEncodedLen`)
	fn withdraw_offer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2222`
		//  Estimated: `4087`
		// Minimum execution time: 71_555_000 picoseconds.
		Weight::from_parts(71_998_000, 4087)
			.saturating_add(RocksDbWeight::get().reads(6_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: `Uniques::Asset` (r:1 w:1)
	/// Proof: `Uniques::Asset` (`max_values`: None, `max_size`: Some(146), added: 2621, mode: `MaxEncodedLen`)
	/// Storage: `Marketplace::Offers` (r:1 w:1)
	/// Proof: `Marketplace::Offers` (`max_values`: None, `max_size`: Some(148), added: 2623, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::NextAssetId` (r:1 w:0)
	/// Proof: `AssetRegistry::NextAssetId` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::LocationAssets` (r:1 w:0)
	/// Proof: `AssetRegistry::LocationAssets` (`max_values`: None, `max_size`: Some(622), added: 3097, mode: `MaxEncodedLen`)
	/// Storage: `Tokens::Accounts` (r:2 w:2)
	/// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(108), added: 2583, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::Assets` (r:1 w:0)
	/// Proof: `AssetRegistry::Assets` (`max_values`: None, `max_size`: Some(87), added: 2562, mode: `MaxEncodedLen`)
	/// Storage: `Marketplace::Prices` (r:1 w:1)
	/// Proof: `Marketplace::Prices` (`max_values`: None, `max_size`: Some(80), added: 2555, mode: `MaxEncodedLen`)
	/// Storage: `Marketplace::MarketplaceItems` (r:1 w:0)
	/// Proof: `Marketplace::MarketplaceItems` (`max_values`: None, `max_size`: Some(98), added: 2573, mode: `MaxEncodedLen`)
	/// Storage: `NFT::Collections` (r:1 w:0)
	/// Proof: `NFT::Collections` (`max_values`: None, `max_size`: Some(99), added: 2574, mode: `MaxEncodedLen`)
	/// Storage: `Uniques::Class` (r:1 w:0)
	/// Proof: `Uniques::Class` (`max_values`: None, `max_size`: Some(190), added: 2665, mode: `MaxEncodedLen`)
	/// Storage: `Uniques::Account` (r:0 w:2)
	/// Proof: `Uniques::Account` (`max_values`: None, `max_size`: Some(112), added: 2587, mode: `MaxEncodedLen`)
	/// Storage: `Uniques::ItemPriceOf` (r:0 w:1)
	/// Proof: `Uniques::ItemPriceOf` (`max_values`: None, `max_size`: Some(113), added: 2588, mode: `MaxEncodedLen`)
	fn accept_offer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `2862`
		//  Estimated: `6156`
		// Minimum execution time: 155_222_000 picoseconds.
		Weight::from_parts(156_370_000, 6156)
			.saturating_add(RocksDbWeight::get().reads(11_u64))
			.saturating_add(RocksDbWeight::get().writes(8_u64))
	}
	/// Storage: `Marketplace::MarketplaceItems` (r:1 w:1)
	/// Proof: `Marketplace::MarketplaceItems` (`max_values`: None, `max_size`: Some(98), added: 2573, mode: `MaxEncodedLen`)
	/// Storage: `Uniques::Asset` (r:1 w:0)
	/// Proof: `Uniques::Asset` (`max_values`: None, `max_size`: Some(146), added: 2621, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::NextAssetId` (r:1 w:0)
	/// Proof: `AssetRegistry::NextAssetId` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::LocationAssets` (r:1 w:0)
	/// Proof: `AssetRegistry::LocationAssets` (`max_values`: None, `max_size`: Some(622), added: 3097, mode: `MaxEncodedLen`)
	fn add_royalty() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1643`
		//  Estimated: `4087`
		// Minimum execution time: 52_363_000 picoseconds.
		Weight::from_parts(53_037_000, 4087)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}
