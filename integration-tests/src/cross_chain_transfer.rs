#![cfg(test)]
use crate::kusama_test_net::*;

use frame_support::{assert_noop, assert_ok, traits::{OnFinalize, OnInitialize}};

use pallet_price_oracle::PriceEntry;
use pallet_transaction_multi_payment::Price;
use polkadot_xcm::latest::prelude::*;

use cumulus_primitives_core::ParaId;
use orml_traits::currency::MultiCurrency;
use primitives::asset::AssetPair;
use sp_runtime::traits::AccountIdConversion;
use xcm_emulator::TestExt;
use hydradx_traits::{AMM, pools::SpotPriceProvider};
use primitives::Balance;
use sp_runtime::traits::Zero;
use pallet_xyk::XYKSpotPrice;

fn last_basilisk_events(n: usize) -> Vec<basilisk_runtime::Event> {
	frame_system::Pallet::<basilisk_runtime::Runtime>::events()
		.into_iter()
		.rev()
		.take(n)
		.rev()
		.map(|e| e.event)
		.collect()
}

fn expect_basilisk_events(e: Vec<basilisk_runtime::Event>) {
	assert_eq!(last_basilisk_events(e.len()), e);
}

#[test]
fn transfer_from_relay_chain() {
	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::parent())
		));
	});
	KusamaRelay::execute_with(|| {
		assert_ok!(kusama_runtime::XcmPallet::reserve_transfer_assets(
			kusama_runtime::Origin::signed(ALICE.into()),
			Box::new(Parachain(2000).into().into()),
			Box::new(
				Junction::AccountId32 {
					id: BOB,
					network: NetworkId::Any
				}
				.into()
				.into()
			),
			Box::new((Here, 3 * BSX).into()),
			0,
		));

		assert_eq!(
			kusama_runtime::Balances::free_balance(&ParaId::from(2000).into_account()),
			13 * BSX
		);
	});

	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			3 * BSX
		);
	});
}

#[test]
fn transfer_to_relay_chain() {
	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::parent())
		));

		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::Origin::signed(ALICE.into()),
			1,
			3 * BSX,
			Box::new(
				MultiLocation::new(
					1,
					X1(Junction::AccountId32 {
						id: BOB,
						network: NetworkId::Any,
					})
				)
				.into()
			),
			4_600_000_000
		));
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(ALICE)),
			200 * BSX - 3 * BSX
		);
	});

	KusamaRelay::execute_with(|| {
		assert_eq!(
			kusama_runtime::Balances::free_balance(&AccountId::from(BOB)),
			2999893333340 // 3 * BSX - fee
		);
	});
}

#[test]
fn transfer_from_hydra() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(3000), GeneralKey(vec![0, 0, 0, 0]))))
		));
	});

	Hydra::execute_with(|| {
		assert_ok!(basilisk_runtime::XTokens::transfer(
			basilisk_runtime::Origin::signed(ALICE.into()),
			0,
			3 * BSX,
			Box::new(
				MultiLocation::new(
					1,
					X2(
						Junction::Parachain(2000),
						Junction::AccountId32 {
							id: BOB,
							network: NetworkId::Any,
						}
					)
				)
				.into()
			),
			399_600_000_000
		));
		assert_eq!(
			basilisk_runtime::Balances::free_balance(&AccountId::from(ALICE)),
			200 * BSX - 3 * BSX
		);
	});

	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)),
			3 * BSX
		);
	});
}
#[test]
fn transfer_insufficient_amount_should_fail() {
	TestNet::reset();

	Basilisk::execute_with(|| {
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::Origin::root(),
			1,
			basilisk_runtime::AssetLocation(MultiLocation::new(1, X2(Parachain(3000), GeneralKey(vec![0, 0, 0, 0]))))
		));
	});

	Hydra::execute_with(|| {
		assert_noop!(
			basilisk_runtime::XTokens::transfer(
				basilisk_runtime::Origin::signed(ALICE.into()),
				0,
				1_000_000 - 1,
				Box::new(
					MultiLocation::new(
						1,
						X2(
							Junction::Parachain(2000),
							Junction::AccountId32 {
								id: BOB,
								network: NetworkId::Any,
							}
						)
					)
					.into()
				),
				399_600_000_000
			),
			orml_xtokens::Error::<basilisk_runtime::Runtime>::XcmExecutionFailed
		);
		assert_eq!(
			basilisk_runtime::Balances::free_balance(&AccountId::from(ALICE)),
			200000000000000
		);
	});

	Basilisk::execute_with(|| {
		// Xcm should fail therefore nothing should be deposit into beneficiary account
		assert_eq!(basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB)), 0);
	});
}

#[test]
fn non_native_fee_payment_works() {

	// comments Martin

	// try from_inner(18decimals)


	TestNet::reset();

	Basilisk::execute_with(|| {

		let curr_0 = 0;
		let curr_1 = 1;
		let asset_a = 1000;
		let asset_b = 2000;

		// ------------ BOB ------------

		assert_ok!(basilisk_runtime::MultiTransactionPayment::set_currency(
			basilisk_runtime::Origin::signed(BOB.into()),
			curr_1,
		));

		let bob_bal = basilisk_runtime::Tokens::free_balance(1, &AccountId::from(BOB));

		// How to calculate and verify the remaining 462_676_500_000 spent on fees?
		assert_eq!(bob_bal, 999_537_323_500_000);

		let pair_account = basilisk_runtime::XYK::get_pair_id(AssetPair {
			asset_in: curr_0,
			asset_out: curr_1,
		});
	
		let share_token = basilisk_runtime::XYK::share_token(pair_account.clone());
		
		// assert_ok!(basilisk_runtime::Tokens::set_balance(
		// 	basilisk_runtime::Origin::root(),
		// 	pair_account.clone().into(),
		// 	1,
		// 	1_000 * BSX,
		// 	0,
		// ));

		// assert_ok!(basilisk_runtime::Balances::set_balance(
		// 	basilisk_runtime::Origin::root(),
		// 	pair_account.clone().into(),
		// 	1_000 * BSX,
		// 	0,
		// ));

		// How does this work and how exactly it affects price?
		assert_ok!(basilisk_runtime::XYK::create_pool(
			basilisk_runtime::Origin::signed(ALICE.into()),
			curr_0, // 1000 BSX 
			curr_1, // 500 KSM 500_000_033_400_002
			1_000 * BSX,
			Price::from_float(0.5),
		));

		let spot_price = XYKSpotPrice::<basilisk_runtime::Runtime>::spot_price(curr_0, curr_1); // = 2

		// Why XYK buy does not affect price?
		assert_ok!(basilisk_runtime::XYK::buy(
			basilisk_runtime::Origin::signed(ALICE.into()),
			0,
			1,
			66 * BSX,
			1_000 * BSX,
			false,
		));

		assert_eq!(basilisk_runtime::XYK::get_pool_assets(&pair_account), Some(vec![curr_0, curr_1]));

		let asset_a_reserve = basilisk_runtime::Currencies::free_balance(curr_0, &pair_account);
		let asset_b_reserve = basilisk_runtime::Currencies::free_balance(curr_1, &pair_account);

		

		println!("pair_account: {:?}", pair_account);
		println!("share_token: {:?}", share_token);
		println!("asset_a_reserve: {:?}", asset_a_reserve);
		println!("asset_b_reserve: {:?}", asset_b_reserve);
		println!("spot_price: {:?}", spot_price);	

		let price_from_1 = Price::from_inner(1);
		println!("price_from_1: {:?}", price_from_1);

		let next = basilisk_runtime::AssetRegistry::next_asset_id();
		println!("next: {:?}", next);	
		
		// ------------ DAVE ------------
		
		assert_ok!(basilisk_runtime::MultiTransactionPayment::set_currency(
			basilisk_runtime::Origin::signed(DAVE.into()),
			curr_1,
		));

		let dave_bal = basilisk_runtime::Tokens::free_balance(1, &AccountId::from(DAVE));
		//assert_eq!(dave_bal, 999_537_323_499_910);

		let expected_diff = 90;
		//assert_eq!(dave_bal, bob_bal - expected_diff);

		// Is this correct oracle setup?
		pub const PRICE_ENTRY_1: PriceEntry = PriceEntry {
			price: Price::from_inner(5000000000000000000),
			trade_amount: 1_000,
			liquidity_amount: 2_000,
		};

		for i in 1..11 {
            basilisk_runtime::PriceOracle::on_initialize(i);
            basilisk_runtime::System::set_block_number(i.into());
            basilisk_runtime::PriceOracle::on_trade(curr_0, curr_1, PRICE_ENTRY_1);
            basilisk_runtime::PriceOracle::on_finalize(i.into());
        }

		let pair_name = basilisk_runtime::PriceOracle::get_name(curr_0, curr_1);
		// println!("pair_name: {:?}", pair_name);
		let data_ten = basilisk_runtime::PriceOracle::price_data_ten();
		let data_thousand = basilisk_runtime::PriceOracle::price_data_thousand(pair_name.clone());
		println!("data_thousand: {:?}", data_thousand);
		//println!("data_thousand: {:?}", data_thousand);

		expect_basilisk_events(vec![
			pallet_transaction_multi_payment::Event::FeeWithdrawn(
				DAVE.into(),
				1,
				462_676_500_000,
				231_338_280_875,
				FALLBACK.into(),
			)
			.into(),
			pallet_transaction_multi_payment::Event::CurrencySet(
				DAVE.into(),
				1,
			)
			.into(),
		]);


	});
}