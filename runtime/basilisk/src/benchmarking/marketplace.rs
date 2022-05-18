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
use crate::{Marketplace, Runtime, Uniques, NFT};

use frame_benchmarking::account;
use frame_support::{traits::Currency, BoundedVec};
use frame_system::RawOrigin;
use orml_benchmarking::runtime_benchmarks;
use sp_runtime::{
	traits::{StaticLookup, UniqueSaturatedInto},
	SaturatedConversion,
};
use sp_std::convert::TryInto;

type BoundedVecOfUnq = pallet_nft::BoundedVecOfUnq<Runtime>;
type NftClassId = <Runtime as pallet_nft::Config>::NftClassId;
type NftInstanceId = <Runtime as pallet_nft::Config>::NftInstanceId;
type Lookup = <Runtime as frame_system::Config>::Lookup;
type Offer = pallet_marketplace::OfferOf<Runtime>;

const SEED: u32 = 0;
const ENDOWMENT: u32 = 1_000_000;
const CLASS_ID_0: u32 = 1_000_000;
const INSTANCE_ID_0: u32 = 0;

fn create_funded_account(name: &'static str, index: u32) -> AccountId {
	let caller: AccountId = account(name, index, SEED);

	let amount = unit(ENDOWMENT);
	drop(<Runtime as pallet_nft::Config>::Currency::deposit_creating(
		&caller,
		amount.unique_saturated_into(),
	));

	caller
}

fn unit(d: u32) -> u128 {
	let d: u128 = d.into();
	d.saturating_mul(1_000_000_000_000)
}

fn create_class_and_mint(
	class_id: NftClassId,
	instance_id: NftInstanceId,
) -> (AccountId, AccountId, <Lookup as StaticLookup>::Source, BoundedVecOfUnq) {
	let caller = create_funded_account("caller", 0);
	let caller2 = create_funded_account("caller2", 1);

	let caller_lookup = Lookup::unlookup(caller.clone());
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

	assert!(NFT::mint(
		RawOrigin::Signed(caller.clone()).into(),
		class_id,
		instance_id,
		metadata.clone()
	)
	.is_ok());
	(caller, caller2, caller_lookup, metadata)
}

runtime_benchmarks! {
	{ Runtime, pallet_marketplace }

	buy {
		let (caller, caller2, caller_lookup, metadata) = create_class_and_mint(CLASS_ID_0.into(), INSTANCE_ID_0.into());
		Marketplace::set_price(RawOrigin::Signed(caller).into(), CLASS_ID_0.into(), INSTANCE_ID_0.into(), Some(u32::max_value().into()))?;
	}: _(RawOrigin::Signed(caller2.clone()), CLASS_ID_0.into(), INSTANCE_ID_0.into())
	verify {
		assert_eq!(Uniques::owner(NftClassId::from(CLASS_ID_0), NftInstanceId::from(INSTANCE_ID_0)), Some(caller2))
	}

	set_price {
		let (caller, caller2, caller_lookup, metadata) = create_class_and_mint(CLASS_ID_0.into(), INSTANCE_ID_0.into());
	}: _(RawOrigin::Signed(caller.clone()), CLASS_ID_0.into(), INSTANCE_ID_0.into(), Some(u32::max_value().into()))
	verify {
		assert_eq!(Marketplace::prices(NftClassId::from(CLASS_ID_0), NftInstanceId::from(INSTANCE_ID_0)), Some(u32::max_value().into()))
	}

	make_offer {
		let (caller, caller2, caller_lookup, metadata) = create_class_and_mint(CLASS_ID_0.into(), INSTANCE_ID_0.into());
	}: _(RawOrigin::Signed(caller.clone()), CLASS_ID_0.into(), INSTANCE_ID_0.into(), unit(100_000).saturated_into(), 666u32)
	verify {
		assert_eq!(
			Marketplace::offers((NftClassId::from(CLASS_ID_0), NftInstanceId::from(INSTANCE_ID_0)), caller.clone()),
			Some( Offer {maker: caller, amount: unit(100_000).saturated_into(), expires: 666u32})
		)
	}

	withdraw_offer {
		let caller2 = create_funded_account("caller2", 0);
		let (caller, caller2, caller_lookup, metadata) = create_class_and_mint(CLASS_ID_0.into(), INSTANCE_ID_0.into());
		Marketplace::make_offer(RawOrigin::Signed(caller2.clone()).into(), CLASS_ID_0.into(), INSTANCE_ID_0.into(), unit(100_000).saturated_into(), 666u32)?;
	}: _(RawOrigin::Signed(caller2.clone()), CLASS_ID_0.into(), INSTANCE_ID_0.into(), caller2.clone())
	verify {
		assert_eq!(
			Marketplace::offers((NftClassId::from(CLASS_ID_0), NftInstanceId::from(INSTANCE_ID_0)), caller2),
			None
		)
	}

	accept_offer {
		let caller2 = create_funded_account("caller2", 0);
		let (caller, caller2, caller_lookup, metadata) = create_class_and_mint(CLASS_ID_0.into(), INSTANCE_ID_0.into());
		Marketplace::make_offer(RawOrigin::Signed(caller2.clone()).into(), CLASS_ID_0.into(), INSTANCE_ID_0.into(), unit(100_000).saturated_into(), 666u32)?;
	}: _(RawOrigin::Signed(caller), CLASS_ID_0.into(), INSTANCE_ID_0.into(), caller2.clone())
	verify {
		assert_eq!(
			Marketplace::offers((NftClassId::from(CLASS_ID_0), NftInstanceId::from(INSTANCE_ID_0)), caller2),
			None
		)
	}

	add_royalty {
		let caller2 = create_funded_account("caller2", 0);
		let (caller, caller2, caller_lookup, metadata) = create_class_and_mint(CLASS_ID_0.into(), INSTANCE_ID_0.into());
	}: _(RawOrigin::Signed(caller), CLASS_ID_0.into(), INSTANCE_ID_0.into(), caller2, 25u8)
	verify {
		assert!(
			pallet_marketplace::MarketplaceInstances::<Runtime>::contains_key(NftClassId::from(CLASS_ID_0), NftInstanceId::from(INSTANCE_ID_0))
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
