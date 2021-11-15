// This file is part of HydraDX.

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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod benchmarking;
pub mod weights;

use frame_support::{dispatch::DispatchResult, traits::Contains, traits::Get};

use orml_traits::{
	arithmetic::{Signed, SimpleArithmetic},
	GetByKey, MultiCurrency, MultiCurrencyExtended,
};

use frame_system::{ensure_root, ensure_signed};

use sp_std::convert::{TryFrom, TryInto};

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::weights::WeightInfo;
	use frame_support::pallet_prelude::*;
	use frame_support::sp_runtime::traits::AtLeast32BitUnsigned;
	use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn blacklisted)]
	/// Accounts excluded from dusting.
	pub type AccountBlacklist<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn reward_account)]
	/// Account to take reward from.
	pub type RewardAccount<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn dust_dest_account)]
	/// Account to send dust to.
	pub type DustAccount<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Balance type
		type Balance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen;

		/// The amount type, should be signed version of `Balance`
		type Amount: Signed
			+ TryInto<Self::Balance>
			+ TryFrom<Self::Balance>
			+ Parameter
			+ Member
			+ SimpleArithmetic
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize;

		/// Asset type
		type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord + From<u32>;

		/// Currency for transfers
		type MultiCurrency: MultiCurrencyExtended<
			Self::AccountId,
			CurrencyId = Self::CurrencyId,
			Balance = Self::Balance,
			Amount = Self::Amount,
		>;

		/// The minimum amount required to keep an account.
		type MinCurrencyDeposits: GetByKey<Self::CurrencyId, Self::Balance>;

		/// Reward amount
		#[pallet::constant]
		type Reward: Get<Self::Balance>;

		/// Native Asset Id
		#[pallet::constant]
		type NativeCurrencyId: Get<Self::CurrencyId>;

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub account_blacklist: Vec<T::AccountId>,
		pub reward_account: T::AccountId,
		pub dust_account: T::AccountId,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				account_blacklist: vec![],
				reward_account: Default::default(),
				dust_account: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			self.account_blacklist.iter().for_each(|account_id| {
				AccountBlacklist::<T>::insert(account_id, ());
			});

			if self.reward_account == Default::default() {
				panic!("Reward account is not set in genesis config");
			}

			if self.dust_account == Default::default() {
				panic!("Dust account is not set in genesis config");
			}

			RewardAccount::<T>::put(&self.reward_account);
			DustAccount::<T>::put(&self.dust_account);
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Account is excluded from dusting.
		AccountBlacklisted,

		/// Account is not present in the non-dustable list.
		AccountNotBlacklisted,

		/// The balance is zero.
		ZeroBalance,

		/// The balance is sufficient to keep account open.
		BalanceSufficient,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Account dusted.
		Dusted(T::AccountId, T::Balance),

		/// Account added to non-dustable list.
		Added(T::AccountId),

		/// Account removed from non-dustable list.
		Removed(T::AccountId),
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Dust specified account.
		/// IF account balance is < min. existential deposit of given currency, and account is allowed to
		/// be dusted, the remaining balance is transferred to selected account (usually treasury).
		///
		/// Caller is rewarded with chosen reward in native currency.
		#[pallet::weight((<T as Config>::WeightInfo::dust_account(), DispatchClass::Normal, Pays::Yes))]
		pub fn dust_account(origin: OriginFor<T>, account: T::AccountId, currency_id: T::CurrencyId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::blacklisted(&account).is_none(), Error::<T>::AccountBlacklisted);

			let (dustable, dust) = Self::is_dustable(&account, currency_id);

			ensure!(dust != T::Balance::from(0u32), Error::<T>::ZeroBalance);

			ensure!(dustable, Error::<T>::BalanceSufficient);

			Self::transfer_dust(&account, &Self::dust_dest_account(), currency_id, dust)?;

			Self::deposit_event(Event::Dusted(account, dust));

			// Ignore the result, it fails - no problem.
			let _ = Self::reward_duster(&who, currency_id, dust);

			Ok(())
		}

		/// Add account to list of non-dustable account. Account whihc are excluded from udsting.
		/// If such account should be dusted - `AccountBlacklisted` error is returned.
		/// Only root can perform this action.
		#[pallet::weight((<T as Config>::WeightInfo::add_nondustable_account(), DispatchClass::Normal, Pays::No))]
		pub fn add_nondustable_account(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			AccountBlacklist::<T>::insert(&account, ());

			Self::deposit_event(Event::Added(account));

			Ok(())
		}

		/// Remove account from list of non-dustable accounts. That means account can be dusted again.
		#[pallet::weight((<T as Config>::WeightInfo::remove_nondustable_account(), DispatchClass::Normal, Pays::No))]
		pub fn remove_nondustable_account(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			AccountBlacklist::<T>::mutate(&account, |maybe_account| -> DispatchResult {
				ensure!(!maybe_account.is_none(), Error::<T>::AccountNotBlacklisted);

				*maybe_account = None;

				Ok(())
			})?;

			Self::deposit_event(Event::Removed(account));

			Ok(())
		}
	}
}
impl<T: Config> Pallet<T> {
	/// Check is account's balance is below minimum deposit.
	fn is_dustable(account: &T::AccountId, currency_id: T::CurrencyId) -> (bool, T::Balance) {
		let ed = T::MinCurrencyDeposits::get(&currency_id);

		let total = T::MultiCurrency::total_balance(currency_id, account);

		(total < ed, total)
	}

	/// Send reward to account which did the dusting.
	fn reward_duster(_duster: &T::AccountId, _currency_id: T::CurrencyId, _dust: T::Balance) -> DispatchResult {
		let reserve_account = Self::reward_account();
		let reward = T::Reward::get();

		T::MultiCurrency::transfer(T::NativeCurrencyId::get(), &reserve_account, _duster, reward)?;

		Ok(())
	}

	/// Transfer dust amount to selected DustAccount ( usually treasury)
	fn transfer_dust(
		from: &T::AccountId,
		dest: &T::AccountId,
		currency_id: T::CurrencyId,
		dust: T::Balance,
	) -> DispatchResult {
		T::MultiCurrency::transfer(currency_id, from, dest, dust)
	}
}

use orml_traits::OnDust;

use sp_std::marker::PhantomData;
pub struct DusterWhitelist<T>(PhantomData<T>);

impl<T: Config> OnDust<T::AccountId, T::CurrencyId, T::Balance> for Pallet<T> {
	fn on_dust(who: &T::AccountId, currency_id: T::CurrencyId, amount: T::Balance) {
		let _ = Self::transfer_dust(who, &Self::dust_dest_account(), currency_id, amount);
	}
}

impl<T: Config> Contains<T::AccountId> for DusterWhitelist<T> {
	fn contains(t: &T::AccountId) -> bool {
		AccountBlacklist::<T>::contains_key(t)
	}
}
