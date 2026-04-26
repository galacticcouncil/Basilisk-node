use crate::{mock::*, Event};
use frame_system::Pallet as System;
use pallet_session::SessionManager;

fn last_bench_event() -> Option<(AccountId, sp_staking::SessionIndex)> {
	System::<Test>::events().into_iter().rev().find_map(|r| match r.event {
		RuntimeEvent::CollatorRotation(Event::CollatorBenched { who, session }) => Some((who, session)),
		_ => None,
	})
}

fn bench_events() -> Vec<(AccountId, sp_staking::SessionIndex)> {
	System::<Test>::events()
		.into_iter()
		.filter_map(|r| match r.event {
			RuntimeEvent::CollatorRotation(Event::CollatorBenched { who, session }) => Some((who, session)),
			_ => None,
		})
		.collect()
}

#[test]
fn benches_index_zero_in_session_zero() {
	ExtBuilder.build().execute_with(|| {
		set_inner(Some(vec![1, 2, 3, 4, 5]));
		let out = <crate::Pallet<Test> as SessionManager<AccountId>>::new_session(0).unwrap();
		assert_eq!(out, vec![2, 3, 4, 5]);
		assert_eq!(last_bench_event(), Some((1, 0)));
	});
}

#[test]
fn benches_rotates_across_sessions() {
	ExtBuilder.build().execute_with(|| {
		let set = vec![10, 20, 30, 40, 50];
		set_inner(Some(set.clone()));

		// Each member benched exactly once over `len` consecutive sessions.
		let mut benched = Vec::new();
		for idx in 0..set.len() as u32 {
			let out = <crate::Pallet<Test> as SessionManager<AccountId>>::new_session(idx).unwrap();
			assert_eq!(out.len(), set.len() - 1);
			benched.push(set[idx as usize % set.len()]);
		}

		let mut events: Vec<AccountId> = bench_events().into_iter().map(|(w, _)| w).collect();
		events.sort();
		let mut expected = set.clone();
		expected.sort();
		assert_eq!(events, expected);
	});
}

#[test]
fn bench_index_wraps_modulo_set_length() {
	ExtBuilder.build().execute_with(|| {
		let set = vec![10, 20, 30];
		set_inner(Some(set.clone()));
		// session 7 -> 7 % 3 = 1 -> bench 20
		let out = <crate::Pallet<Test> as SessionManager<AccountId>>::new_session(7).unwrap();
		assert_eq!(out, vec![10, 30]);
		assert_eq!(last_bench_event(), Some((20, 7)));
	});
}

#[test]
fn does_not_bench_single_collator_set() {
	ExtBuilder.build().execute_with(|| {
		set_inner(Some(vec![42]));
		let out = <crate::Pallet<Test> as SessionManager<AccountId>>::new_session(0).unwrap();
		assert_eq!(out, vec![42]);
		assert!(bench_events().is_empty());
	});
}

#[test]
fn propagates_none_from_inner() {
	ExtBuilder.build().execute_with(|| {
		set_inner(None);
		let out = <crate::Pallet<Test> as SessionManager<AccountId>>::new_session(0);
		assert!(out.is_none());
		assert!(bench_events().is_empty());
	});
}

#[test]
fn empty_set_from_inner_passes_through_without_bench() {
	ExtBuilder.build().execute_with(|| {
		set_inner(Some(vec![]));
		let out = <crate::Pallet<Test> as SessionManager<AccountId>>::new_session(0).unwrap();
		assert!(out.is_empty());
		assert!(bench_events().is_empty());
	});
}

#[test]
fn end_and_start_session_pass_through() {
	ExtBuilder.build().execute_with(|| {
		<crate::Pallet<Test> as SessionManager<AccountId>>::end_session(3);
		<crate::Pallet<Test> as SessionManager<AccountId>>::end_session(4);
		<crate::Pallet<Test> as SessionManager<AccountId>>::start_session(7);
		assert_eq!(end_calls(), vec![3, 4]);
		assert_eq!(start_calls(), vec![7]);
	});
}
