//      ---_ ......._-_--.        ,adPPYba, 8b,dPPYba,    ,adPPYba,  88   ,d8
//     (|\ /      / /| \  \       I8[    "" 88P'   `"8a  a8P_____88  88 ,a8"
//     /  /     .'  -=-'   `.      `"Y8ba,  88       88  8PP"""""""  8888[
//    /  /    .'             )    aa    ]8I 88       88  "8b,   ,aa  88`"Yba,
//  _/  /   .'        _.)   /     `"YbbdP"' 88       88   `"Ybbd8"'  88   `Y8a
//  / o   o        _.-' /  .'
//  \          _.-'    / .'*|
//  \______.-'//    .'.' \*|      This file is part of Basilisk-node.
//   \|  \ | //   .'.' _ |*|      Built with <3 for decentralisation.
//    `   \|//  .'.'_ _ _|*|
//     .  .// .'.' | _ _ \*|      Copyright (C) 2021-2022  Intergalactic, Limited (GIB).
//     \`-|\_/ /    \ _ _ \*\     SPDX-License-Identifier: Apache-2.0
//      `/'\__/      \ _ _ \*\    Licensed under the Apache License, Version 2.0 (the "License");
//     /^|            \ _ _ \*    you may not use this file except in compliance with the License.
//    '  `             \ _ _ \    http://www.apache.org/licenses/LICENSE-2.0
//     '  `             \ _ _ \

//! Autogenerated weights for pallet_auctions
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-06-06, STEPS: 1, REPEAT: 1, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: None, WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 1024

// Executed Command:
// target/release/basilisk
// benchmark
// --extrinsic
// *
// --pallet
// pallet-auctions
// --output
// ./pallets/auctions/src/weights.rs
// --template=.maintain/pallet-weight-template.hbs
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(clippy::unnecessary_cast)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_auctions.
pub trait WeightInfo {
	fn create_english() -> Weight;
	fn update_english() -> Weight;
	fn destroy_english() -> Weight;
	fn bid_english() -> Weight;
	fn close_english() -> Weight;
	fn create_topup() -> Weight;
	fn update_topup() -> Weight;
	fn destroy_topup() -> Weight;
	fn bid_topup() -> Weight;
	fn close_topup() -> Weight;
	fn claim_topup() -> Weight;
	fn create_candle() -> Weight;
	fn update_candle() -> Weight;
	fn destroy_candle() -> Weight;
	fn bid_candle() -> Weight;
	fn close_candle() -> Weight;
	fn claim_candle() -> Weight;
}

pub struct BasiliskWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for BasiliskWeight<T> {
	fn create_english() -> Weight {
		Weight::from_ref_time(26_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	fn update_english() -> Weight {
		Weight::from_ref_time(9_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	fn destroy_english() -> Weight {
		Weight::from_ref_time(25_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	fn bid_english() -> Weight {
		Weight::from_ref_time(35_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	fn close_english() -> Weight {
		Weight::from_ref_time(51_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(7 as u64))
	}
	fn create_topup() -> Weight {
		Weight::from_ref_time(23_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	fn update_topup() -> Weight {
		Weight::from_ref_time(16_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	fn destroy_topup() -> Weight {
		Weight::from_ref_time(68_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	fn bid_topup() -> Weight {
		Weight::from_ref_time(33_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	fn close_topup() -> Weight {
		Weight::from_ref_time(53_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(6 as u64))
			.saturating_add(T::DbWeight::get().writes(7 as u64))
	}
	fn claim_topup() -> Weight {
		Weight::from_ref_time(35_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	fn create_candle() -> Weight {
		Weight::from_ref_time(25_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(4 as u64))
	}
	fn update_candle() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(2 as u64))
			.saturating_add(T::DbWeight::get().writes(1 as u64))
	}
	fn destroy_candle() -> Weight {
		Weight::from_ref_time(26_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(3 as u64))
			.saturating_add(T::DbWeight::get().writes(3 as u64))
	}
	fn bid_candle() -> Weight {
		Weight::from_ref_time(32_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(5 as u64))
			.saturating_add(T::DbWeight::get().writes(5 as u64))
	}
	fn close_candle() -> Weight {
		Weight::from_ref_time(58_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(9 as u64))
			.saturating_add(T::DbWeight::get().writes(7 as u64))
	}
	fn claim_candle() -> Weight {
		Weight::from_ref_time(34_000_000 as u64)
			.saturating_add(T::DbWeight::get().reads(4 as u64))
			.saturating_add(T::DbWeight::get().writes(6 as u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	fn create_english() -> Weight {
		Weight::from_ref_time(26_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
	fn update_english() -> Weight {
		Weight::from_ref_time(9_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	fn destroy_english() -> Weight {
		Weight::from_ref_time(25_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	fn bid_english() -> Weight {
		Weight::from_ref_time(35_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
	fn close_english() -> Weight {
		Weight::from_ref_time(51_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(6 as u64))
			.saturating_add(RocksDbWeight::get().writes(7 as u64))
	}
	fn create_topup() -> Weight {
		Weight::from_ref_time(23_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
	fn update_topup() -> Weight {
		Weight::from_ref_time(16_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	fn destroy_topup() -> Weight {
		Weight::from_ref_time(68_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	fn bid_topup() -> Weight {
		Weight::from_ref_time(33_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
	fn close_topup() -> Weight {
		Weight::from_ref_time(53_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(6 as u64))
			.saturating_add(RocksDbWeight::get().writes(7 as u64))
	}
	fn claim_topup() -> Weight {
		Weight::from_ref_time(35_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
	fn create_candle() -> Weight {
		Weight::from_ref_time(25_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(4 as u64))
	}
	fn update_candle() -> Weight {
		Weight::from_ref_time(10_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(2 as u64))
			.saturating_add(RocksDbWeight::get().writes(1 as u64))
	}
	fn destroy_candle() -> Weight {
		Weight::from_ref_time(26_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(3 as u64))
			.saturating_add(RocksDbWeight::get().writes(3 as u64))
	}
	fn bid_candle() -> Weight {
		Weight::from_ref_time(32_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(5 as u64))
			.saturating_add(RocksDbWeight::get().writes(5 as u64))
	}
	fn close_candle() -> Weight {
		Weight::from_ref_time(58_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(9 as u64))
			.saturating_add(RocksDbWeight::get().writes(7 as u64))
	}
	fn claim_candle() -> Weight {
		Weight::from_ref_time(34_000_000 as u64)
			.saturating_add(RocksDbWeight::get().reads(4 as u64))
			.saturating_add(RocksDbWeight::get().writes(6 as u64))
	}
}