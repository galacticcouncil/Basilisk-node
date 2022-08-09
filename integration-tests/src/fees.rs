#![cfg(test)]

use crate::kusama_test_net::*;
use basilisk_runtime::TransactionPayment;
use frame_support::weights::GetDispatchInfo;
use primitives::nft::ClassType;
use sp_runtime::codec::Encode;
use xcm_emulator::TestExt;

macro_rules! assert_eq_approx {
	( $x:expr, $y:expr, $z:expr, $r:expr) => {{
		let diff = if $x >= $y { $x - $y } else { $y - $x };
		if diff > $z {
			panic!("\n{} not equal\nleft: {:?}\nright: {:?}\n", $r, $x, $y);
		}
	}};
}

//NOTE: rust encoded call size is differen from UI encoded call size that's why we have asserts for 2 fees.

#[test]
fn transaction_fees_should_be_as_expected_when_transfer_happen() {
	Basilisk::execute_with(|| {
		let diff = UNITS / 100; //0.01

		let expected_rust_encoded_fees = 4_556 * UNITS / 100; //45.56
		let expected_ui_fees = 4_655 * UNITS / 100; //46.55

		let call = currencies::Call::<basilisk_runtime::Runtime>::transfer {
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
			"Currencies transfer:\n\t UI fees: {}/{} [actual/expected]\n\t Rust encoded fees: {}/{} [actual/expected]\n\t Fees with min. FeeMultiplier: {} [UI], {} [Rust]",
			format_num(ui_fees * 10_000 / UNITS, 4),
			format_num(expected_ui_fees * 10_000 / UNITS, 4),
			format_num(rust_encoded_fees * 10_000 / UNITS, 4),
			format_num(expected_rust_encoded_fees * 10_000 / UNITS, 4),
			format_num(min_multiplier_ui_fees * 10_000 / UNITS, 4),
			format_num(min_multiplier_rust_fees * 10_000 / UNITS, 4),
		);

		assert_eq_approx!(rust_encoded_fees, expected_rust_encoded_fees, diff, "rust fees changed");
		assert_eq_approx!(ui_fees, expected_ui_fees, diff, "ui fees changed");
	});
}

#[test]
fn transaction_fees_should_be_as_expected_when_nft_is_minted() {
	Basilisk::execute_with(|| {
		//NOTE: Price showed by polkadotAPPS is changing at second decimal place between runs.
		let diff = UNITS / 10; //0.1

		let expected_rust_encoded_fees = 41_130 * UNITS / 100; //411.30
		let expected_ui_fees = 41_233 * UNITS / 100; //412.33

		let call = pallet_nft::Call::<basilisk_runtime::Runtime>::mint {
			class_id: 1_000_000,
			instance_id: 0,
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

		assert_eq_approx!(rust_encoded_fees, expected_rust_encoded_fees, diff, "rust fees changed");
		assert_eq_approx!(ui_fees, expected_ui_fees, diff, "ui fees changed");
	});
}

#[test]
fn transaction_fees_should_be_as_expected_when_nft_class_is_created() {
	Basilisk::execute_with(|| {
		//NOTE: Price showed by polkadotAPPS is changing at second decimal place between runs.
		let diff = UNITS / 10; //0.1

		let expected_rust_encoded_fees = 39_879 * UNITS / 100; //398.79
		let expected_ui_fees = 39_982 * UNITS / 100; //399.82

		let call = pallet_nft::Call::<basilisk_runtime::Runtime>::create_class {
			class_id: 0,
			class_type: ClassType::Marketplace,
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
			"NFT create_class\n\t UI fees: {}/{} [actual/expected]\n\t Rust encoded fees: {}/{} [actual/expected]\n\t Fees with min. FeeMultiplier: {} [UI], {} [Rust]",
			format_num(ui_fees * 10_000 / UNITS, 4),
			format_num(expected_ui_fees * 10_000 / UNITS, 4),
			format_num(rust_encoded_fees * 10_000 / UNITS, 4),
			format_num(expected_rust_encoded_fees * 10_000 / UNITS, 4),
			format_num(min_multiplier_ui_fees * 10_000 / UNITS, 4),
			format_num(min_multiplier_rust_fees * 10_000 / UNITS, 4),
		);

		assert_eq_approx!(rust_encoded_fees, expected_rust_encoded_fees, diff, "rust fees changed");
		assert_eq_approx!(ui_fees, expected_ui_fees, diff, "ui fees changed");
	});
}

fn format_num(num: u128, decimals: usize) -> String {
	let p = num.to_string();

	let split = p.split_at(p.len() - decimals);

	format!("{}.{}", split.0, split.1)
}
