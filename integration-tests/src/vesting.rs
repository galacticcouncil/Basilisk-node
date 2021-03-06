#![cfg(test)]

use crate::kusama_test_net::*;
use basilisk_runtime::{BlockNumber, Vesting};
use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;
use orml_vesting::VestingSchedule;
use sp_core::crypto::AccountId32;
use sp_runtime::DispatchError::BadOrigin;
use xcm_emulator::TestExt;

type AccountId = AccountId32;
type Balance = u128;
type Schedule = VestingSchedule<BlockNumber, Balance>;

fn vesting_schedule() -> Schedule {
	Schedule {
		start: 0,
		period: 1,
		period_count: 3,
		per_period: 100_000,
	}
}

#[test]
fn vested_transfer_should_work_when_sent_from_root() {
	Basilisk::execute_with(|| {
		// Arrange
		let to: AccountId = AccountId::from(BOB);
		let vesting_account: AccountId = vesting_account();

		let vesting_account_balance_before = basilisk_runtime::Balances::free_balance(&vesting_account);
		let to_balance_before = basilisk_runtime::Balances::free_balance(&to);

		// Act
		assert_ok!(Vesting::vested_transfer(
			RawOrigin::Root.into(),
			to.clone(),
			vesting_schedule()
		));

		// Assert
		let vesting_account_balance_after = basilisk_runtime::Balances::free_balance(vesting_account);
		let to_balance_after = basilisk_runtime::Balances::free_balance(to);

		assert_eq!(
			vesting_account_balance_after,
			vesting_account_balance_before.checked_sub(300_000).unwrap()
		);
		assert_eq!(to_balance_after, to_balance_before.checked_add(300_000).unwrap());
	});
}

#[test]
fn vested_transfer_should_fail_when_signed_by_any_account() {
	Basilisk::execute_with(|| {
		let from: AccountId = AccountId::from(ALICE);
		let to: AccountId = AccountId::from(BOB);

		// Assert
		assert_noop!(
			Vesting::vested_transfer(RawOrigin::Signed(from).into(), to, vesting_schedule()),
			BadOrigin
		);
	});
}
