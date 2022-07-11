#![cfg(test)]

use crate::kusama_test_net::*;
use basilisk_runtime::TransactionPayment;
use frame_support::weights::GetDispatchInfo;
use sp_runtime::codec::Encode;
use xcm_emulator::TestExt;

#[test]
#[ignore]
fn transaction_fees() {
	//This test is not correct
	Basilisk::execute_with(|| {
		let one = 1_000_000_000_000;

		let sent_amount = 50 * one;

		let call = orml_currencies::Call::<basilisk_runtime::Runtime>::transfer {
			dest: AccountId::from(BOB),
			currency_id: 0,
			amount: sent_amount,
		};

		let info = call.get_dispatch_info();
		let len = call.using_encoded(|e| e.len()) as u32;

		let fees = TransactionPayment::compute_fee(len, &info, 0);

		//This test is not correct it's approx 1bsx of from real fees
		println!("fees: {:?}", fees);
	});
}
