#![cfg(test)]

use crate::kusama_test_net::*;
use basilisk_runtime::Currencies;
use basilisk_runtime::RuntimeOrigin;
use basilisk_runtime::XYK;
use frame_support::dispatch::GetDispatchInfo;
use frame_support::weights::Weight;
use frame_support::{assert_ok, pallet_prelude::*};
use hydradx_traits::router::AssetPair;
use hydradx_traits::router::PoolType;
use hydradx_traits::router::Trade;
use orml_traits::currency::MultiCurrency;
use polkadot_xcm::{latest::prelude::*, VersionedXcm};
use pretty_assertions::assert_eq;
use primitives::constants::chain::CORE_ASSET_ID;
use sp_runtime::FixedU128;
use xcm_emulator::TestExt;

pub const SELL: bool = true;
pub const BUY: bool = false;

#[test]
fn basilisk_should_swap_assets_when_receiving_from_otherchain_with_sell() {
	//Arrange
	TestNet::reset();

	Basilisk::execute_with(|| {
		register_kar();
		add_currency_price(KAR, FixedU128::from(1));

		assert_ok!(basilisk_runtime::Tokens::deposit(KAR, &CHARLIE.into(), 3000 * UNITS));
		create_xyk_pool(KAR, BSX);
	});

	OtherParachain::execute_with(|| {
		let xcm = craft_exchange_asset_xcm::<_, basilisk_runtime::RuntimeCall>(
			MultiAsset::from((GeneralIndex(0), 5 * UNITS)),
			MultiAsset::from((GeneralIndex(CORE_ASSET_ID.into()), 2 * UNITS)),
			SELL,
		);
		//Act
		let res = basilisk_runtime::PolkadotXcm::execute(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			Box::new(xcm),
			Weight::from_parts(399_600_000_000, 0),
		);
		assert_ok!(res);

		//Assert
		assert_eq!(
			basilisk_runtime::Balances::free_balance(AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN - 100 * UNITS
		);

		assert!(matches!(
			last_other_para_events(2).first(),
			Some(basilisk_runtime::RuntimeEvent::XcmpQueue(
				cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. }
			))
		));
	});

	let fees = 27_500_000_000_000;
	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(KAR, &AccountId::from(BOB)),
			95000000000000 - fees
		);
		let received = BOB_INITIAL_BSX_BALANCE + 2373809523812;
		assert_eq!(basilisk_runtime::Balances::free_balance(AccountId::from(BOB)), received);
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(KAR, &basilisk_runtime::Treasury::account_id()),
			fees
		);
	});
}

#[test]
fn basilisk_should_swap_assets_when_receiving_from_otherchain_with_buy() {
	//Arrange
	TestNet::reset();

	Basilisk::execute_with(|| {
		register_kar();
		add_currency_price(KAR, FixedU128::from(1));

		assert_ok!(basilisk_runtime::Tokens::deposit(KAR, &CHARLIE.into(), 3000 * UNITS));
		assert_ok!(basilisk_runtime::Tokens::deposit(BSX, &CHARLIE.into(), 3000 * UNITS));
		assert_ok!(XYK::create_pool(
			RuntimeOrigin::signed(CHARLIE.into()),
			KAR,
			1000 * UNITS,
			BSX,
			500 * UNITS,
		));
	});

	let amount_out = 20 * UNITS;
	OtherParachain::execute_with(|| {
		let xcm = craft_exchange_asset_xcm::<_, basilisk_runtime::RuntimeCall>(
			MultiAsset::from((GeneralIndex(0), 70 * UNITS)),
			MultiAsset::from((GeneralIndex(CORE_ASSET_ID.into()), amount_out)),
			BUY,
		);
		//Act
		let res = basilisk_runtime::PolkadotXcm::execute(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			Box::new(xcm),
			Weight::from_parts(399_600_000_000, 0),
		);
		assert_ok!(res);

		//Assert
		assert_eq!(
			basilisk_runtime::Balances::free_balance(AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN - 100 * UNITS
		);

		assert!(matches!(
			last_other_para_events(2).first(),
			Some(basilisk_runtime::RuntimeEvent::XcmpQueue(
				cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. }
			))
		));
	});

	let fees = 27_500_000_000_000;
	let amount_in = 41_791_666_666_665;

	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(KAR, &AccountId::from(BOB)),
			100 * UNITS - amount_in - fees
		);

		assert_eq!(
			basilisk_runtime::Balances::free_balance(AccountId::from(BOB)),
			BOB_INITIAL_BSX_BALANCE + amount_out
		);
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(KAR, &basilisk_runtime::Treasury::account_id()),
			fees
		);
	});
}

#[test]
fn basilisk_should_swap_assets_coming_from_karura_when_onchain_route_present() {
	//Arrange
	TestNet::reset();

	Basilisk::execute_with(|| {
		register_kar();
		assert_ok!(basilisk_runtime::Tokens::deposit(KAR, &CHARLIE.into(), 3000 * UNITS));
		create_xyk_pool_with_amounts(KAR, 100000 * UNITS, BSX, 100000 * UNITS);
		create_xyk_pool_with_amounts(BSX, 100000 * UNITS, KSM, 100000 * UNITS);

		//Register KSM location
		assert_ok!(basilisk_runtime::AssetRegistry::set_location(
			basilisk_runtime::RuntimeOrigin::root(),
			KSM,
			basilisk_runtime::AssetLocation(MultiLocation::new(0, X1(GeneralIndex(3))))
		));

		//Register onchain route from KAR to KSM
		assert_ok!(basilisk_runtime::Router::set_route(
			RuntimeOrigin::signed(CHARLIE.into()),
			AssetPair::new(KAR, KSM),
			vec![
				Trade {
					pool: PoolType::XYK,
					asset_in: KAR,
					asset_out: BSX,
				},
				Trade {
					pool: PoolType::XYK,
					asset_in: BSX,
					asset_out: KSM,
				},
			],
		));

		add_currency_price(KAR, FixedU128::from(1));
	});

	OtherParachain::execute_with(|| {
		let xcm = craft_exchange_asset_xcm::<_, basilisk_runtime::RuntimeCall>(
			MultiAsset::from((GeneralIndex(0), 5 * UNITS)),
			MultiAsset::from((GeneralIndex(KSM.into()), 2 * UNITS)),
			SELL,
		);
		//Act
		let res = basilisk_runtime::PolkadotXcm::execute(
			basilisk_runtime::RuntimeOrigin::signed(ALICE.into()),
			Box::new(xcm),
			Weight::from_parts(399_600_000_000, 0),
		);
		assert_ok!(res);

		//Assert
		assert_eq!(
			basilisk_runtime::Balances::free_balance(AccountId::from(ALICE)),
			ALICE_INITIAL_NATIVE_BALANCE_ON_OTHER_PARACHAIN - 100 * UNITS
		);

		assert!(matches!(
			last_other_para_events(2).first(),
			Some(basilisk_runtime::RuntimeEvent::XcmpQueue(
				cumulus_pallet_xcmp_queue::Event::XcmpMessageSent { .. }
			))
		));
	});

	let fees = 27_500_000_000_000;
	Basilisk::execute_with(|| {
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(KAR, &AccountId::from(BOB)),
			95000000000000 - fees
		);
		let received = 4969548790555;
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(KSM, &AccountId::from(BOB)),
			received
		);
		assert_eq!(
			basilisk_runtime::Tokens::free_balance(KAR, &basilisk_runtime::Treasury::account_id()),
			fees
		);
	});
}

fn register_kar() {
	assert_ok!(basilisk_runtime::AssetRegistry::register(
		basilisk_runtime::RuntimeOrigin::root(),
		b"KAR".to_vec(),
		pallet_asset_registry::AssetType::Token,
		1_000_000,
		Some(KAR),
		None,
		Some(basilisk_runtime::AssetLocation(MultiLocation::new(
			1,
			X2(Parachain(OTHER_PARA_ID), GeneralIndex(0))
		))),
		None
	));
}

fn add_currency_price(asset_id: u32, price: FixedU128) {
	assert_ok!(basilisk_runtime::MultiTransactionPayment::add_currency(
		basilisk_runtime::RuntimeOrigin::root(),
		asset_id,
		price,
	));

	// make sure the price is propagated
	basilisk_runtime::MultiTransactionPayment::on_initialize(basilisk_runtime::System::block_number());
}

fn craft_exchange_asset_xcm<M: Into<MultiAssets>, RC: Decode + GetDispatchInfo>(
	give: MultiAsset,
	want: M,
	is_sell: bool,
) -> VersionedXcm<RC> {
	use kusama_runtime::xcm_config::BaseXcmWeight;
	use xcm_builder::FixedWeightBounds;
	use xcm_executor::traits::WeightBounds;

	type Weigher<RC> = FixedWeightBounds<BaseXcmWeight, RC, ConstU32<100>>;

	let dest = MultiLocation::new(1, Parachain(BASILISK_PARA_ID));
	let beneficiary = Junction::AccountId32 { id: BOB, network: None }.into();
	let assets: MultiAssets = MultiAsset::from((GeneralIndex(0), 100 * UNITS)).into(); // hardcoded
	let max_assets = assets.len() as u32 + 1;
	let context = X2(GlobalConsensus(NetworkId::Polkadot), Parachain(OTHER_PARA_ID));
	let fees = assets
		.get(0)
		.expect("should have at least 1 asset")
		.clone()
		.reanchored(&dest, context)
		.expect("should reanchor");
	let give = give.reanchored(&dest, context).expect("should reanchor give");
	let give: MultiAssetFilter = Definite(give.into());
	let want = want.into();
	let weight_limit = {
		let fees = fees.clone();
		let mut remote_message = Xcm(vec![
			ReserveAssetDeposited::<RC>(assets.clone()),
			ClearOrigin,
			BuyExecution {
				fees,
				weight_limit: Limited(Weight::zero()),
			},
			ExchangeAsset {
				give: give.clone(),
				want: want.clone(),
				maximal: is_sell,
			},
			DepositAsset {
				assets: Wild(AllCounted(max_assets)),
				beneficiary,
			},
		]);
		// use local weight for remote message and hope for the best.
		let remote_weight = Weigher::weight(&mut remote_message).expect("weighing should not fail");
		Limited(remote_weight)
	};
	// executed on remote (on hydra)
	let xcm = Xcm(vec![
		BuyExecution { fees, weight_limit },
		ExchangeAsset {
			give,
			want,
			maximal: is_sell,
		},
		DepositAsset {
			assets: Wild(AllCounted(max_assets)),
			beneficiary,
		},
	]);
	// executed on local (acala)
	let message = Xcm(vec![
		SetFeesMode { jit_withdraw: true },
		TransferReserveAsset { assets, dest, xcm },
	]);
	VersionedXcm::V3(message)
}

pub fn last_other_para_events(n: usize) -> Vec<basilisk_runtime::RuntimeEvent> {
	frame_system::Pallet::<basilisk_runtime::Runtime>::events()
		.into_iter()
		.rev()
		.take(n)
		.rev()
		.map(|e| e.event)
		.collect()
}

fn create_xyk_pool(asset_a: u32, asset_b: u32) {
	assert_ok!(XYK::create_pool(
		RuntimeOrigin::signed(CHARLIE.into()),
		asset_a,
		100 * UNITS,
		asset_b,
		50 * UNITS,
	));
}

fn create_xyk_pool_with_amounts(asset_a: u32, amount_a: u128, asset_b: u32, amount_b: u128) {
	assert_ok!(Currencies::update_balance(
		RuntimeOrigin::root(),
		DAVE.into(),
		asset_a,
		amount_a as i128,
	));

	assert_ok!(Currencies::update_balance(
		RuntimeOrigin::root(),
		DAVE.into(),
		asset_b,
		amount_b as i128,
	));
	assert_ok!(XYK::create_pool(
		RuntimeOrigin::signed(DAVE.into()),
		asset_a,
		amount_a,
		asset_b,
		amount_b,
	));
}
