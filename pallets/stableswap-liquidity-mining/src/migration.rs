// Copyright (C) 2020-2022  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;

#[allow(dead_code)]
pub fn init_nft_collection<T: Config>() -> frame_support::weights::Weight {
	//let version = StorageVersion::get::<Pallet<T>>();

	todo!()
}

/*
TODO: fix this tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::Test;
	use frame_support::assert_noop;

	#[test]
	fn init_nft_class_migration_should_work() {
		sp_io::TestExternalities::default().execute_with(|| {
			init_nft_class::<Test>();

			let pallet_account = <Pallet<Test>>::account_id();

			assert_noop!(
				pallet_nft::Pallet::<Test>::do_create_class(
					pallet_account,
					mock::LIQ_MINING_NFT_CLASS,
					ClassType::LiquidityMining,
					vec![].try_into().unwrap(),
				),
				pallet_uniques::Error::<Test>::InUse
			);
		});
	}

	#[test]
	fn second_migration_should_do_nothing_work() {
		sp_io::TestExternalities::default().execute_with(|| {
			init_nft_class::<Test>();

			let pallet_account = <Pallet<Test>>::account_id();

			assert_noop!(
				pallet_nft::Pallet::<Test>::do_create_class(
					pallet_account,
					mock::LIQ_MINING_NFT_CLASS,
					ClassType::LiquidityMining,
					vec![].try_into().unwrap(),
				),
				pallet_uniques::Error::<Test>::InUse
			);

			init_nft_class::<Test>();
		});
	}
}
*/
