#![cfg(test)]

use crate::kusama_test_net::*;
use basilisk_runtime::TransactionPayment;
use frame_support::weights::GetDispatchInfo;
use sp_runtime::codec::Encode;
use xcm_emulator::TestExt;

#[test]
// #[ignore]
//TODO: fix this, fees calculcated in this test are apporx 2bsx off.
fn transfer_transaction_fees() {
	//This test is not correct
	Basilisk::execute_with(|| {
		let call = orml_currencies::Call::<basilisk_runtime::Runtime>::transfer {
			dest: AccountId::from(ALICE),
			currency_id: 0,
			amount: 50 * UNITS,
		};

		let info = call.get_dispatch_info();
		let len = call.encoded_size() as u32;
		let fees = TransactionPayment::compute_fee(len, &info, 0);

		//This test is not correct it's approx 1bsx of from real fees
		println!("transfer cost {:?} BSX", fees / UNITS);
	});
}

#[test]
fn nft_mint_transaction_fees() {
	Basilisk::execute_with(|| {
		let call = pallet_nft::Call::<basilisk_runtime::Runtime>::mint {
			class_id: 0,
			instance_id: 0,
			metadata: b"ipfs://QmQu2jUmtFNPd86tEHFs6hmAArKYyjEC3xuwVWpFGjcMgm"
				.to_vec()
				.try_into()
				.unwrap(),
		};

		let info = call.get_dispatch_info();
		let len = call.encoded_size() as u32;
		let fees = TransactionPayment::compute_fee(len, &info, 0);

		//This test is not correct it's approx 1bsx of from real fees
		println!("mint nft cost {:?} BSX", fees / UNITS);
	});
}
