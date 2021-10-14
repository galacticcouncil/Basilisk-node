// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
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

//! Various pieces of common functionality.

use super::*;
use crate::traits::InstanceReserve;
use frame_support::{ensure, traits::Get, BoundedVec};
use sp_runtime::DispatchResult;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	pub fn set_instance_attribute(
		class: T::ClassId,
		instance: T::InstanceId,
		key: BoundedVec<u8, T::KeyLimit>,
		value: BoundedVec<u8, T::ValueLimit>,
	) -> DispatchResult {
		let mut class_details = Class::<T, I>::get(&class).ok_or(Error::<T, I>::Unknown)?;
		let instance_details = Asset::<T, I>::get(&class, &instance).ok_or(Error::<T, I>::Unknown)?;

		let is_frozen = InstanceMetadataOf::<T, I>::get(class, instance).map(|v| v.is_frozen);

		ensure!(!is_frozen.unwrap_or(false), Error::<T, I>::Frozen);

		let attribute = Attribute::<T, I>::get((class, Some(instance), &key));

		if attribute.is_none() {
			class_details.attributes.saturating_inc();
		}

		let old_deposit = attribute.map_or(Zero::zero(), |m| m.1);

		class_details.total_deposit.saturating_reduce(old_deposit);

		let mut deposit = Zero::zero();

		if !class_details.free_holding {
			deposit = T::DepositPerByte::get()
				.saturating_mul(((key.len() + value.len()) as u32).into())
				.saturating_add(T::AttributeDepositBase::get());
		}
		class_details.total_deposit.saturating_accrue(deposit);

		if deposit > old_deposit {
			T::InstanceReserveStrategy::reserve::<T, I>(
				&instance_details.owner,
				&class_details.owner,
				&class_details.admin,
				&class_details.issuer,
				deposit - old_deposit,
			)?;
		} else if deposit < old_deposit {
			T::InstanceReserveStrategy::unreserve::<T, I>(
				&instance_details.owner,
				&class_details.owner,
				&class_details.admin,
				&class_details.issuer,
				old_deposit - deposit,
			)?;
		}

		Attribute::<T, I>::insert((&class, Some(instance), &key), (&value, deposit));
		Class::<T, I>::insert(class, &class_details);
		Self::deposit_event(Event::AttributeSet(class, Some(instance), key, value));

		Ok(())
	}
}
