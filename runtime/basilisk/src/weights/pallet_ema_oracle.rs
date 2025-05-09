// This file is part of Basilisk.

// Copyright (C) 2020-2023  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


//! Autogenerated weights for `pallet_ema_oracle`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 43.0.0
//! DATE: 2025-01-15, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `bench-bot`, CPU: `Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `None`, DB CACHE: `1024`

// Executed Command:
// ./target/release/basilisk
// benchmark
// pallet
// --wasm-execution=compiled
// --pallet
// pallet-ema-oracle
// --extrinsic
// *
// --heap-pages
// 4096
// --steps
// 50
// --repeat
// 20
// --template=scripts/pallet-weight-template.hbs
// --output
// runtime/basilisk/src/weights/pallet_ema_oracle.rs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weights for `pallet_ema_oracle`.
pub struct WeightInfo<T>(PhantomData<T>);

/// Weights for `pallet_ema_oracle` using the Basilisk node and recommended hardware.
pub struct BasiliskWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_ema_oracle::WeightInfo for BasiliskWeight<T> {
	/// Storage: `EmaOracle::WhitelistedAssets` (r:1 w:1)
	/// Proof: `EmaOracle::WhitelistedAssets` (`max_values`: Some(1), `max_size`: Some(481), added: 976, mode: `MaxEncodedLen`)
	fn add_oracle() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `601`
		//  Estimated: `1966`
		// Minimum execution time: 17_736_000 picoseconds.
		Weight::from_parts(17_933_000, 1966)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `EmaOracle::WhitelistedAssets` (r:1 w:1)
	/// Proof: `EmaOracle::WhitelistedAssets` (`max_values`: Some(1), `max_size`: Some(481), added: 976, mode: `MaxEncodedLen`)
	/// Storage: `EmaOracle::Accumulator` (r:1 w:1)
	/// Proof: `EmaOracle::Accumulator` (`max_values`: Some(1), `max_size`: Some(4441), added: 4936, mode: `MaxEncodedLen`)
	/// Storage: `EmaOracle::Oracles` (r:0 w:5)
	/// Proof: `EmaOracle::Oracles` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
	fn remove_oracle() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `617`
		//  Estimated: `5926`
		// Minimum execution time: 43_174_000 picoseconds.
		Weight::from_parts(43_759_000, 5926)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(7_u64))
	}
	/// Storage: `EmaOracle::Accumulator` (r:1 w:0)
	/// Proof: `EmaOracle::Accumulator` (`max_values`: Some(1), `max_size`: Some(4441), added: 4936, mode: `MaxEncodedLen`)
	fn on_finalize_no_entry() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `109`
		//  Estimated: `5926`
		// Minimum execution time: 4_402_000 picoseconds.
		Weight::from_parts(4_705_000, 5926)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	/// Storage: `EmaOracle::Accumulator` (r:1 w:1)
	/// Proof: `EmaOracle::Accumulator` (`max_values`: Some(1), `max_size`: Some(4441), added: 4936, mode: `MaxEncodedLen`)
	/// Storage: `EmaOracle::Oracles` (r:145 w:145)
	/// Proof: `EmaOracle::Oracles` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
	/// The range of component `b` is `[1, 29]`.
	fn on_finalize_multiple_tokens(b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `204 + b * (933 ±0)`
		//  Estimated: `5926 + b * (13260 ±0)`
		// Minimum execution time: 75_744_000 picoseconds.
		Weight::from_parts(13_812_323, 5926)
			// Standard Error: 22_668
			.saturating_add(Weight::from_parts(61_141_685, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().reads((5_u64).saturating_mul(b.into())))
			.saturating_add(T::DbWeight::get().writes(1_u64))
			.saturating_add(T::DbWeight::get().writes((5_u64).saturating_mul(b.into())))
			.saturating_add(Weight::from_parts(0, 13260).saturating_mul(b.into()))
	}
	/// Storage: `EmaOracle::Accumulator` (r:1 w:1)
	/// Proof: `EmaOracle::Accumulator` (`max_values`: Some(1), `max_size`: Some(4441), added: 4936, mode: `MaxEncodedLen`)
	/// The range of component `b` is `[1, 29]`.
	fn on_trade_multiple_tokens(b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `204 + b * (148 ±0)`
		//  Estimated: `5926`
		// Minimum execution time: 8_850_000 picoseconds.
		Weight::from_parts(8_871_191, 5926)
			// Standard Error: 1_727
			.saturating_add(Weight::from_parts(447_164, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `EmaOracle::Accumulator` (r:1 w:1)
	/// Proof: `EmaOracle::Accumulator` (`max_values`: Some(1), `max_size`: Some(4441), added: 4936, mode: `MaxEncodedLen`)
	/// The range of component `b` is `[1, 29]`.
	fn on_liquidity_changed_multiple_tokens(b: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `204 + b * (148 ±0)`
		//  Estimated: `5926`
		// Minimum execution time: 8_859_000 picoseconds.
		Weight::from_parts(8_869_447, 5926)
			// Standard Error: 1_740
			.saturating_add(Weight::from_parts(448_576, 0).saturating_mul(b.into()))
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: `EmaOracle::Oracles` (r:2 w:0)
	/// Proof: `EmaOracle::Oracles` (`max_values`: None, `max_size`: Some(177), added: 2652, mode: `MaxEncodedLen`)
	fn get_entry() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `609`
		//  Estimated: `6294`
		// Minimum execution time: 21_890_000 picoseconds.
		Weight::from_parts(22_197_000, 6294)
			.saturating_add(T::DbWeight::get().reads(2_u64))
	}
}