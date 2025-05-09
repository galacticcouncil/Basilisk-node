#![cfg(test)]

use crate::kusama_test_net::*;
use basilisk_runtime::TransactionPayment;
use frame_support::dispatch::GetDispatchInfo;
use pallet_nft::CollectionType;
use sp_runtime::codec::Encode;
use xcm_emulator::TestExt;

macro_rules! assert_eq_approx {
	( $x:expr, $y:expr, $z:expr, $r:expr) => {{
		let d = if $x >= $y { $x - $y } else { $y - $x };
		if d > $z {
			panic!(
				"\n{}\nleft:  {:?}\nright: {:?}\nallowed diff: {:?}\nactual diff:  {:?}\n",
				$r, $x, $y, $z, d
			);
		}
	}};
}

//NOTE: rust encoded call size is different from UI encoded call size that's why we have asserts for 2 fees.

const DIFF: u128 = 100 * UNITS;

#[test]
fn transaction_fees_should_be_as_expected_when_transfer_happen() {
	Basilisk::execute_with(|| {
		let expected_rust_encoded_fees = 4_705 * UNITS / 100; //47.05
		let expected_ui_fees = 4_804 * UNITS / 100; //48.04

		let call = pallet_currencies::Call::<basilisk_runtime::Runtime>::transfer {
			dest: AccountId::from(ALICE),
			currency_id: 0,
			amount: 50 * UNITS,
		};

		let info = call.get_dispatch_info();
		//rust encoded fees
		let rust_encoded_len = call.encoded_size() as u32;
		let rust_encoded_fees = TransactionPayment::compute_fee(rust_encoded_len, &info, 0);

		//UI encoded fees
		let ui_encoded_len = 143;
		let encoding_difference = 99;
		assert_eq!(
			ui_encoded_len - rust_encoded_len,
			encoding_difference,
			"ui encoding difference changed"
		);
		let ui_fees = TransactionPayment::compute_fee(ui_encoded_len, &info, 0);

		// Min fee adjustment multiplier
		pallet_transaction_payment::pallet::NextFeeMultiplier::<basilisk_runtime::Runtime>::put(
			basilisk_runtime::MinimumMultiplier::get(),
		);
		let min_multiplier_rust_fees = TransactionPayment::compute_fee(rust_encoded_len, &info, 0);
		let min_multiplier_ui_fees = TransactionPayment::compute_fee(ui_encoded_len, &info, 0);

		println!(
			"Pallet currencies transfer:\n\t UI fees: {}/{} [actual/expected]\n\t Rust encoded fees: {}/{} [actual/expected]\n\t Fees with min. FeeMultiplier: {} [UI], {} [Rust]",
			format_num(ui_fees * 10_000 / UNITS, 4),
			format_num(expected_ui_fees * 10_000 / UNITS, 4),
			format_num(rust_encoded_fees * 10_000 / UNITS, 4),
			format_num(expected_rust_encoded_fees * 10_000 / UNITS, 4),
			format_num(min_multiplier_ui_fees * 10_000 / UNITS, 4),
			format_num(min_multiplier_rust_fees * 10_000 / UNITS, 4),
		);

		assert_eq_approx!(rust_encoded_fees, expected_rust_encoded_fees, DIFF, "rust fees changed");
		assert_eq_approx!(ui_fees, expected_ui_fees, DIFF, "ui fees changed");
	});
}

#[test]
fn transaction_fees_should_be_as_expected_when_nft_is_minted() {
	Basilisk::execute_with(|| {
		let expected_rust_encoded_fees = 94_764 * UNITS / 100; //947.64
		let expected_ui_fees = 94_869 * UNITS / 100; //948.69

		let call = pallet_nft::Call::<basilisk_runtime::Runtime>::mint {
			collection_id: 1_000_000,
			item_id: 0,
			metadata: b"ipfs://QmQu2jUmtFNPd86tEHFs6hmAArKYyjEC3xuwVWpFGjcMgm"
				.to_vec()
				.try_into()
				.unwrap(),
		};

		let info = call.get_dispatch_info();
		//rust encoded fees
		let rust_encoded_len = call.encoded_size() as u32;
		let rust_encoded_fees = TransactionPayment::compute_fee(rust_encoded_len, &info, 0);

		//UI encoded fees
		let ui_encoded_len = 192;
		let encoding_difference = 105;
		assert_eq!(
			ui_encoded_len - rust_encoded_len,
			encoding_difference,
			"ui encoding difference changed"
		);
		let ui_fees = TransactionPayment::compute_fee(ui_encoded_len, &info, 0);

		// Min fee adjustment multiplier
		pallet_transaction_payment::pallet::NextFeeMultiplier::<basilisk_runtime::Runtime>::put(
			basilisk_runtime::MinimumMultiplier::get(),
		);

		let min_multiplier_rust_fees = TransactionPayment::compute_fee(rust_encoded_len, &info, 0);
		let min_multiplier_ui_fees = TransactionPayment::compute_fee(ui_encoded_len, &info, 0);

		println!(
			"NFT mint:\n\t UI fees: {}/{} [actual/expected]\n\t Rust encoded fees: {}/{} [actual/expected]\n\t Fees with min. FeeMultiplier: {} [UI], {} [Rust]",
			format_num(ui_fees * 10_000 / UNITS, 4),
			format_num(expected_ui_fees * 10_000 / UNITS, 4),
			format_num(rust_encoded_fees * 10_000 / UNITS, 4),
			format_num(expected_rust_encoded_fees * 10_000 / UNITS, 4),
			format_num(min_multiplier_ui_fees * 10_000 / UNITS, 4),
			format_num(min_multiplier_rust_fees * 10_000 / UNITS, 4),
		);

		// before trying to fix this test, make sure that CREATE_COLLECTION_OFFSET and MINT_OFFSET
		// were added to the rebenchmarked weights.
		assert_eq_approx!(
			rust_encoded_fees,
			expected_rust_encoded_fees,
			DIFF,
			"rust fees difference too large"
		);
		assert_eq_approx!(ui_fees, expected_ui_fees, DIFF, "ui fees difference too large");
	});
}

#[test]
fn transaction_fees_should_be_as_expected_when_nft_collection_is_created() {
	Basilisk::execute_with(|| {
		let expected_rust_encoded_fees = 70_000 * UNITS / 100;
		let expected_ui_fees = 70_000 * UNITS / 100;

		let call = pallet_nft::Call::<basilisk_runtime::Runtime>::create_collection {
			collection_id: 0,
			collection_type: CollectionType::Marketplace,
			metadata: b"ipfs://QmQu2jUmtFNPd86tEHFs6hmAArKYyjEC3xuwVWpFGjcMgm"
				.to_vec()
				.try_into()
				.unwrap(),
		};

		let info = call.get_dispatch_info();
		//rust encoded fees
		let rust_encoded_len = call.encoded_size() as u32;
		let rust_encoded_fees = TransactionPayment::compute_fee(rust_encoded_len, &info, 0);

		//UI encoded fees
		let ui_encoded_len = 177;
		let encoding_difference = 105;
		assert_eq!(
			ui_encoded_len - rust_encoded_len,
			encoding_difference,
			"ui encoding difference changed"
		);
		let ui_fees = TransactionPayment::compute_fee(ui_encoded_len, &info, 0);

		// Min fee adjustment multiplier
		pallet_transaction_payment::pallet::NextFeeMultiplier::<basilisk_runtime::Runtime>::put(
			basilisk_runtime::MinimumMultiplier::get(),
		);

		let min_multiplier_rust_fees = TransactionPayment::compute_fee(rust_encoded_len, &info, 0);
		let min_multiplier_ui_fees = TransactionPayment::compute_fee(ui_encoded_len, &info, 0);

		println!(
			"NFT create_collection\n\t UI fees: {}/{} [actual/expected]\n\t Rust encoded fees: {}/{} [actual/expected]\n\t Fees with min. FeeMultiplier: {} [UI], {} [Rust]",
			format_num(ui_fees * 10_000 / UNITS, 4),
			format_num(expected_ui_fees * 10_000 / UNITS, 4),
			format_num(rust_encoded_fees * 10_000 / UNITS, 4),
			format_num(expected_rust_encoded_fees * 10_000 / UNITS, 4),
			format_num(min_multiplier_ui_fees * 10_000 / UNITS, 4),
			format_num(min_multiplier_rust_fees * 10_000 / UNITS, 4),
		);

		assert_eq_approx!(rust_encoded_fees, expected_rust_encoded_fees, DIFF, "rust fees changed");
		assert_eq_approx!(ui_fees, expected_ui_fees, DIFF, "ui fees changed");
	});
}

fn format_num(num: u128, decimals: usize) -> String {
	let p = num.to_string();

	let split = p.split_at(p.len() - decimals);

	format!("{}.{}", split.0, split.1)
}
