// This file is part of Basilisk-node.

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

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use crate::Pallet as CollatorRewards;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, account};
use sp_runtime::testing::UintAuthorityId;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use frame_support::codec::Decode;
use pallet_session as session;

const SEED: u32 = 0;

fn keys<T: Config + session::Config>(c: u32) -> <T as session::Config>::Keys {
	use rand::{RngCore, SeedableRng};

	let keys = {
		let mut keys = [0u8; 128];

		if c > 0 {
			let mut rng = rand::rngs::StdRng::seed_from_u64(c as u64);
			rng.fill_bytes(&mut keys);
		}

		keys
	};

	Decode::decode(&mut &keys[..]).unwrap()
}

benchmarks! {
    where_clause{ where T: pallet_session::Config}

    on_new_session {
        let who: T::AccountId = account("c1", 1, SEED);

    use rand::{RngCore, SeedableRng};

    let keys = keys::<T>(1);

        let collators = vec![(&who, keys)];
    }: {
        CollatorRewards::on_new_session(true, collators.into_iter(), vec![].into_iter()) 
        
    } verify {
        assert!(false);
    }
}

#[cfg(test)]
mod tests {
	use super::Pallet;
    use frame_benchmarking::impl_benchmark_test_suite;

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
