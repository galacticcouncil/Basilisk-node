// This file is part of Basilisk-node

// Copyright (C) 2020-2021  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{vec, AccountId};
use crate::{Runtime, Uniques, NFT};

use frame_benchmarking::account;
use frame_support::{
	traits::{
		tokens::nonfungibles::{Inspect, InspectEnumerable},
		Currency,
	},
	BoundedVec,
};
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use sp_runtime::traits::{StaticLookup, UniqueSaturatedInto};
use sp_std::convert::TryInto;

use pallet_nft::BoundedVecOfUnq;
type NftClassId = <Runtime as pallet_nft::Config>::NftClassId;
type NftInstanceId = <Runtime as pallet_nft::Config>::NftInstanceId;

const SEED: u32 = 0;
const ENDOWMENT: u32 = 1_000_000;
const CLASS_ID_0: u32 = 1_000_000;

fn create_funded_account(name: &'static str, index: u32) -> AccountId {
	let caller: AccountId = account(name, index, SEED);

	let amount = dollar(ENDOWMENT);
	drop(<Runtime as pallet_nft::Config>::Currency::deposit_creating(
		&caller,
		amount.unique_saturated_into(),
	));

	caller
}

fn dollar(d: u32) -> u128 {
	let d: u128 = d.into();
	d.saturating_mul(primitives::constants::currency::DOLLARS)
}

fn do_create_class(
	class_id: NftClassId,
) -> (
	AccountId,
	<<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source,
	BoundedVecOfUnq<Runtime>,
) {
	let caller = create_funded_account("caller", 0);
	let caller_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller.clone());
	let metadata: BoundedVec<_, _> = vec![0; <Runtime as pallet_uniques::Config>::StringLimit::get() as usize]
		.try_into()
		.unwrap();
	assert!(NFT::create_class(
		RawOrigin::Signed(caller.clone()).into(),
		class_id,
		Default::default(),
		metadata.clone()
	)
	.is_ok());
	(caller, caller_lookup, metadata)
}

fn do_mint(class_id: NftClassId, instance_id: NftInstanceId) {
	let caller = create_funded_account("caller", 0);
	let metadata: BoundedVec<_, _> = vec![0; <Runtime as pallet_uniques::Config>::StringLimit::get() as usize]
		.try_into()
		.unwrap();
	assert!(NFT::mint(RawOrigin::Signed(caller).into(), class_id, instance_id, metadata).is_ok());
}

runtime_benchmarks! {
	{ Runtime, pallet_nft }

	create_class {
		let caller = create_funded_account("caller", 0);
		let metadata: BoundedVec<_, _> = vec![0; <Runtime as pallet_uniques::Config>::StringLimit::get() as usize].try_into().unwrap();
	}: _(RawOrigin::Signed(caller.clone()), CLASS_ID_0.into(), Default::default(), metadata)
	verify {
		assert_eq!(Uniques::class_owner(&NftClassId::from(CLASS_ID_0)), Some(caller));
	}

	mint {
		let (caller, caller_lookup, metadata) = do_create_class(1_000_000u32.into());
		let metadata: BoundedVec<_, _> = vec![0; <Runtime as pallet_uniques::Config>::StringLimit::get() as usize].try_into().unwrap();
	}: _(RawOrigin::Signed(caller.clone()), CLASS_ID_0.into(), 0u32.into(), metadata)
	verify {
		assert_eq!(Uniques::owner(NftClassId::from(CLASS_ID_0), NftInstanceId::from(0u32)), Some(caller));
	}

	transfer {
		let caller2 = create_funded_account("caller2", 1);
		let caller2_lookup = <Runtime as frame_system::Config>::Lookup::unlookup(caller2.clone());
		let (caller, caller_lookup, metadata) = do_create_class(CLASS_ID_0.into());
		do_mint(CLASS_ID_0.into(), 0u32.into());
	}: _(RawOrigin::Signed(caller), CLASS_ID_0.into(), 0u32.into(), caller2_lookup)
	verify {
		assert_eq!(Uniques::owner(NftClassId::from(CLASS_ID_0), NftInstanceId::from(0u32)), Some(caller2));
	}

	destroy_class {
		let (caller, caller_lookup, metadata) = do_create_class(CLASS_ID_0.into());
	}: _(RawOrigin::Signed(caller.clone()), CLASS_ID_0.into())
	verify {
		assert_eq!(Uniques::classes().count(), 0);
	}

	burn {
		let (caller, caller_lookup, metadata) = do_create_class(CLASS_ID_0.into());
		do_mint(CLASS_ID_0.into(), 0u32.into());
	}: _(RawOrigin::Signed(caller.clone()), CLASS_ID_0.into(), 0u32.into())
	verify {
		assert_eq!(Uniques::owned(&caller).count(), 0);
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
