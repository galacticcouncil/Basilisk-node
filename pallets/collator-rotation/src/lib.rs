// This file is part of Basilisk-node.
//
// Copyright (C) 2020-2026  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

use pallet_session::SessionManager;
use sp_staking::SessionIndex;
use sp_std::vec::Vec;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {
		/// The inner session manager whose collator set we wrap.
		type Inner: SessionManager<Self::AccountId>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Announces that `who` will be excluded from the active set in the
		/// upcoming session `session_index`. Emitted from
		/// `SessionManager::new_session`, which pallet-session calls one
		/// session ahead of activation, so this fires one session before the
		/// bench is reflected in `session.validators()`. The field name
		/// matches `session.NewSession`.
		CollatorBenched {
			who: T::AccountId,
			session_index: SessionIndex,
		},
	}
}

impl<T: Config> SessionManager<T::AccountId> for Pallet<T> {
	fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		let mut collators = T::Inner::new_session(new_index)?;
		// Skip benching when the set would otherwise become empty (single-collator session
		// would stall the chain). This is a panic/stall guard, not a configurable floor.
		if collators.len() > 1 {
			let bench_idx = (new_index as usize) % collators.len();
			let benched = collators.remove(bench_idx);
			Self::deposit_event(Event::CollatorBenched {
				who: benched,
				session_index: new_index,
			});
		}
		Some(collators)
	}

	fn end_session(end_index: SessionIndex) {
		T::Inner::end_session(end_index)
	}

	fn start_session(start_index: SessionIndex) {
		T::Inner::start_session(start_index)
	}
}
