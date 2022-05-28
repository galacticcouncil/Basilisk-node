#![cfg(test)]
use crate::kusama_test_net::*;

use frame_support::{
	assert_ok,
	traits::{OnFinalize, OnInitialize},
};

use pallet_price_oracle::{BucketQueueT, PriceInfo};
use pallet_transaction_multi_payment::Price;

use hydradx_traits::{pools::SpotPriceProvider, AMM};
use orml_traits::currency::MultiCurrency;
use pallet_xyk::XYKSpotPrice;
use polkadot_primitives::v1::BlockNumber;
use primitives::asset::AssetPair;
use xcm_emulator::TestExt;

pub fn basilisk_run_to_block(to: BlockNumber) {
	while basilisk_runtime::System::block_number() < to {
		let b = basilisk_runtime::System::block_number();

		basilisk_runtime::System::on_finalize(b);
		basilisk_runtime::PriceOracle::on_finalize(b);
		basilisk_runtime::MultiTransactionPayment::on_finalize(b);

		basilisk_runtime::System::on_initialize(b + 1);
		basilisk_runtime::PriceOracle::on_finalize(b);
		basilisk_runtime::MultiTransactionPayment::on_initialize(b + 1);

		basilisk_runtime::System::set_block_number(b + 1);
	}
}

#[test]
fn non_native_fee_payment_works() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		let currency_0 = 0;
		let currency_1 = 1;

		// ------------ BOB ------------
		assert_ok!(basilisk_runtime::MultiTransactionPayment::set_currency(
			basilisk_runtime::Origin::signed(BOB.into()),
			currency_1,
		));

		let bob_balance = basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB));

		// 462_676_500_000 (~0.46 UNITS) spent on fee
		assert_eq!(bob_balance, 999_537_323_500_000);

		let pair_account = basilisk_runtime::XYK::get_pair_id(AssetPair {
			asset_in: currency_0,
			asset_out: currency_1,
		});

		assert_ok!(basilisk_runtime::Balances::set_balance(
			basilisk_runtime::Origin::root(),
			ALICE.into(),
			2_000_000_000_000 * BSX,
			0,
		));

		assert_ok!(basilisk_runtime::Tokens::set_balance(
			basilisk_runtime::Origin::root(),
			ALICE.into(),
			1,
			2_000_000_000_000 * BSX,
			0,
		));

		assert_ok!(basilisk_runtime::XYK::create_pool(
			basilisk_runtime::Origin::signed(ALICE.into()),
			currency_0, // 1000 BSX
			currency_1, // 500 KSM (500_000_033_400_002)
			1_000 * BSX,
			Price::from_float(0.5),
		));

		let spot_price = XYKSpotPrice::<basilisk_runtime::Runtime>::spot_price(currency_0, currency_1);
		assert_eq!(spot_price, Some(Price::from_float(0.5)));

		basilisk_run_to_block(2);

		assert_ok!(basilisk_runtime::XYK::buy(
			basilisk_runtime::Origin::signed(ALICE.into()),
			0,
			1,
			66 * BSX,
			1_000 * BSX,
			false,
		));

		basilisk_run_to_block(3);

		assert_eq!(
			basilisk_runtime::XYK::get_pool_assets(&pair_account),
			Some(vec![currency_0, currency_1])
		);

		// ------------ DAVE ------------
		assert_ok!(basilisk_runtime::MultiTransactionPayment::set_currency(
			basilisk_runtime::Origin::signed(DAVE.into()),
			currency_1,
		));

		let dave_balance = basilisk_runtime::Tokens::free_balance(1, &AccountId::from(DAVE));
		let expected_diff = 197_453_601_724;
		assert_eq!(dave_balance, bob_balance + expected_diff);

		expect_basilisk_events(vec![
			pallet_transaction_multi_payment::Event::FeeWithdrawn(
				DAVE.into(),
				1,
				462_676_500_000,
				265_222_898_276,
				basilisk_runtime::Treasury::account_id(),
			)
			.into(),
			pallet_transaction_multi_payment::Event::CurrencySet(DAVE.into(), 1).into(),
		]);

		basilisk_run_to_block(11);

		let pair_name = basilisk_runtime::PriceOracle::get_name(currency_0, currency_1);
		let data_ten = basilisk_runtime::PriceOracle::price_data_ten()
			.iter()
			.find(|&x| x.0 == pair_name)
			.unwrap()
			.1;

		assert_eq!(
			data_ten.get_last(),
			PriceInfo {
				avg_price: Price::from_inner(535331905781590000),
				volume: 35_331_905_781_585
			}
		);
	});
}
