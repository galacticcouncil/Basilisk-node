use frame_support::weights::{constants::RocksDbWeight, Weight};

/// Weight functions needed for lbp.
pub trait WeightInfo {
	fn register() -> Weight;
	fn update() -> Weight;
	fn set_metadata() -> Weight;
	fn set_location() -> Weight;
}

/// Weights for `pallet_asset_registry`.
impl WeightInfo for () {
	/// Storage: `AssetRegistry::AssetIds` (r:1 w:1)
	/// Proof: `AssetRegistry::AssetIds` (`max_values`: None, `max_size`: Some(53), added: 2528, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::NextAssetId` (r:1 w:1)
	/// Proof: `AssetRegistry::NextAssetId` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::LocationAssets` (r:1 w:1)
	/// Proof: `AssetRegistry::LocationAssets` (`max_values`: None, `max_size`: Some(622), added: 3097, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::AssetLocations` (r:0 w:1)
	/// Proof: `AssetRegistry::AssetLocations` (`max_values`: None, `max_size`: Some(614), added: 3089, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::AssetMetadataMap` (r:0 w:1)
	/// Proof: `AssetRegistry::AssetMetadataMap` (`max_values`: None, `max_size`: Some(46), added: 2521, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::Assets` (r:0 w:1)
	/// Proof: `AssetRegistry::Assets` (`max_values`: None, `max_size`: Some(87), added: 2562, mode: `MaxEncodedLen`)
	fn register() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `338`
		//  Estimated: `4087`
		// Minimum execution time: 42_033_000 picoseconds.
		Weight::from_parts(42_375_000, 4087)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(6_u64))
	}
	/// Storage: `AssetRegistry::Assets` (r:1 w:1)
	/// Proof: `AssetRegistry::Assets` (`max_values`: None, `max_size`: Some(87), added: 2562, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::AssetIds` (r:1 w:2)
	/// Proof: `AssetRegistry::AssetIds` (`max_values`: None, `max_size`: Some(53), added: 2528, mode: `MaxEncodedLen`)
	fn update() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `405`
		//  Estimated: `3552`
		// Minimum execution time: 29_416_000 picoseconds.
		Weight::from_parts(29_727_000, 3552)
			.saturating_add(RocksDbWeight::get().reads(2_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
	/// Storage: `AssetRegistry::Assets` (r:1 w:0)
	/// Proof: `AssetRegistry::Assets` (`max_values`: None, `max_size`: Some(87), added: 2562, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::AssetMetadataMap` (r:0 w:1)
	/// Proof: `AssetRegistry::AssetMetadataMap` (`max_values`: None, `max_size`: Some(46), added: 2521, mode: `MaxEncodedLen`)
	fn set_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `333`
		//  Estimated: `3552`
		// Minimum execution time: 21_739_000 picoseconds.
		Weight::from_parts(22_133_000, 3552)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: `AssetRegistry::Assets` (r:1 w:0)
	/// Proof: `AssetRegistry::Assets` (`max_values`: None, `max_size`: Some(87), added: 2562, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::LocationAssets` (r:1 w:1)
	/// Proof: `AssetRegistry::LocationAssets` (`max_values`: None, `max_size`: Some(622), added: 3097, mode: `MaxEncodedLen`)
	/// Storage: `AssetRegistry::AssetLocations` (r:1 w:1)
	/// Proof: `AssetRegistry::AssetLocations` (`max_values`: None, `max_size`: Some(614), added: 3089, mode: `MaxEncodedLen`)
	fn set_location() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `307`
		//  Estimated: `4087`
		// Minimum execution time: 26_543_000 picoseconds.
		Weight::from_parts(26_957_000, 4087)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
}
