// This file is part of Basilisk-node.

// Copyright (C) 2020-2023  Intergalactic, Limited (GIB).
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use crate::governance::{SuperMajorityCouncilOrRoot, SuperMajorityTechCommitteeOrRoot, UnanimousTechCommitteeOrRoot};
use crate::system::NativeAssetId;

use hydradx_traits::{
	router::{inverse_route, AmmTradeWeights, PoolType, Trade},
	AssetPairAccountIdFor, LockedBalance, OnTradeHandler, OraclePeriod, Source,
};
use pallet_currencies::fungibles::FungibleCurrencies;
use pallet_currencies::BasicCurrencyAdapter;
use pallet_lbp::weights::WeightInfo as LbpWeights;
use pallet_route_executor::weights::WeightInfo as RouterWeights;
use pallet_transaction_multi_payment::{AddTxAssetOnAccount, RemoveTxAssetOnKilled};
use pallet_xyk::weights::WeightInfo as XykWeights;
use primitives::constants::{
	chain::{DISCOUNTED_FEE, MAX_IN_RATIO, MAX_OUT_RATIO, MIN_POOL_LIQUIDITY, MIN_TRADING_LIMIT},
	currency::{NATIVE_EXISTENTIAL_DEPOSIT, UNITS},
};

use frame_support::{
	parameter_types,
	sp_runtime::{app_crypto::sp_core::crypto::UncheckedFrom, traits::Zero},
	traits::{
		AsEnsureOriginWithArg, Contains, Currency, Defensive, EnsureOrigin, Get, Imbalance, LockIdentifier,
		NeverEnsureOrigin, OnUnbalanced,
	},
	BoundedVec, PalletId,
};
use frame_system::RawOrigin;
use orml_tokens::CurrencyAdapter;
use orml_traits::currency::MutationHooks;

pub struct RelayChainAssetId;
impl Get<AssetId> for RelayChainAssetId {
	fn get() -> AssetId {
		let invalid_id = pallet_asset_registry::Pallet::<Runtime>::next_asset_id().defensive_unwrap_or(AssetId::MAX);

		match pallet_asset_registry::Pallet::<Runtime>::location_to_asset(RELAY_CHAIN_ASSET_LOCATION) {
			Some(asset_id) => asset_id,
			None => invalid_id,
		}
	}
}

type KusamaCurrency = CurrencyAdapter<Runtime, RelayChainAssetId>;

parameter_types! {
	pub const NativeExistentialDeposit: u128 = NATIVE_EXISTENTIAL_DEPOSIT;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

// pallet-treasury did not impl OnUnbalanced<Credit>, need an adapter to handle dust.
type CreditOf = frame_support::traits::fungible::Credit<<Runtime as frame_system::Config>::AccountId, Balances>;
type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;
pub struct DustRemovalAdapter;
impl OnUnbalanced<CreditOf> for DustRemovalAdapter {
	fn on_nonzero_unbalanced(amount: CreditOf) {
		let new_amount = NegativeImbalance::new(amount.peek());
		Treasury::on_nonzero_unbalanced(new_amount);
	}
}

impl pallet_balances::Config for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	type DustRemoval = DustRemovalAdapter;
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous event type.
	type ExistentialDeposit = NativeExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = weights::pallet_balances::BasiliskWeight<Runtime>;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
}

pub struct CurrencyHooks;
impl MutationHooks<AccountId, AssetId, Balance> for CurrencyHooks {
	type OnDust = Duster;
	type OnSlash = ();
	type PreDeposit = ();
	type PostDeposit = ();
	type PreTransfer = ();
	type PostTransfer = ();
	type OnNewTokenAccount = AddTxAssetOnAccount<Runtime>;
	type OnKilledTokenAccount = RemoveTxAssetOnKilled<Runtime>;
}

pub struct DustRemovalWhitelist;
impl Contains<AccountId> for DustRemovalWhitelist {
	fn contains(a: &AccountId) -> bool {
		get_all_module_accounts().contains(a) || pallet_duster::DusterWhitelist::<Runtime>::contains(a)
	}
}

impl orml_tokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = AssetId;
	type WeightInfo = weights::orml_tokens::BasiliskWeight<Runtime>;
	type ExistentialDeposits = AssetRegistry;
	type CurrencyHooks = CurrencyHooks;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ();
	type DustRemovalWhitelist = DustRemovalWhitelist;
}

// The latest versions of the orml-currencies pallet don't emit events.
// The infrastructure relies on the events from this pallet, so we use the latest version of
// the pallet that contains and emit events and was updated to the polkadot version we use.
impl pallet_currencies::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = NativeAssetId;
	type WeightInfo = weights::pallet_currencies::BasiliskWeight<Runtime>;
}

parameter_types! {
	pub const SequentialIdOffset: u32 = 1_000_000;
}
impl pallet_asset_registry::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RegistryOrigin = SuperMajorityTechCommitteeOrRoot;
	type AssetId = AssetId;
	type Balance = Balance;
	type AssetNativeLocation = AssetLocation;
	type StringLimit = RegistryStrLimit;
	type SequentialIdStartAt = SequentialIdOffset;
	type NativeAssetId = NativeAssetId;
	type WeightInfo = weights::pallet_asset_registry::BasiliskWeight<Runtime>;
}

parameter_types! {
	pub const DustingReward: u128 = 0;
}

impl pallet_duster::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = AssetId;
	type MultiCurrency = Currencies;
	type MinCurrencyDeposits = AssetRegistry;
	type Reward = DustingReward;
	type NativeCurrencyId = NativeAssetId;
	type BlacklistUpdateOrigin = MajorityTechCommitteeOrRoot;
	type WeightInfo = weights::pallet_duster::BasiliskWeight<Runtime>;
}

pub struct AssetPairAccountId<T: frame_system::Config>(PhantomData<T>);
impl<T: frame_system::Config> AssetPairAccountIdFor<AssetId, T::AccountId> for AssetPairAccountId<T>
where
	T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
	fn from_assets(asset_a: AssetId, asset_b: AssetId, identifier: &str) -> T::AccountId {
		let mut buf: Vec<u8> = identifier.as_bytes().to_vec();

		if asset_a < asset_b {
			buf.extend_from_slice(&asset_a.to_le_bytes());
			buf.extend_from_slice(&asset_b.to_le_bytes());
		} else {
			buf.extend_from_slice(&asset_b.to_le_bytes());
			buf.extend_from_slice(&asset_a.to_le_bytes());
		}
		T::AccountId::unchecked_from(<T::Hashing as frame_support::sp_runtime::traits::Hash>::hash(&buf[..]))
	}
}

parameter_types! {
	pub ExchangeFee: (u32, u32) = (3, 1_000);
	pub const MinTradingLimit: Balance = MIN_TRADING_LIMIT;
	pub const MinPoolLiquidity: Balance = MIN_POOL_LIQUIDITY;
	pub const MaxInRatio: u128 = MAX_IN_RATIO;
	pub const MaxOutRatio: u128 = MAX_OUT_RATIO;
	pub const RegistryStrLimit: u32 = 32;
	pub const DiscountedFee: (u32, u32) = DISCOUNTED_FEE;
	pub const XYKOracleSourceIdentifier: Source = *b"snek/xyk";
}

impl pallet_xyk::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AssetRegistry = AssetRegistry;
	type AssetPairAccountId = AssetPairAccountId<Self>;
	type Currency = Currencies;
	type NativeAssetId = NativeAssetId;
	type WeightInfo = weights::pallet_xyk::BasiliskWeight<Runtime>; //TODO: add benchmakrs
	type GetExchangeFee = ExchangeFee;
	type MinTradingLimit = MinTradingLimit;
	type MinPoolLiquidity = MinPoolLiquidity;
	type MaxInRatio = MaxInRatio;
	type MaxOutRatio = MaxOutRatio;
	type OracleSource = XYKOracleSourceIdentifier;
	type CanCreatePool = hydradx_adapters::xyk::AllowPoolCreation<Runtime, AssetRegistry>;
	type AMMHandler = pallet_ema_oracle::OnActivityHandler<Runtime>;
	type DiscountedFee = DiscountedFee;
	type NonDustableWhitelistHandler = Duster;
}

pub struct MultiCurrencyLockedBalance<T>(PhantomData<T>);

impl<T: orml_tokens::Config + pallet_balances::Config + frame_system::Config>
	LockedBalance<AssetId, T::AccountId, Balance> for MultiCurrencyLockedBalance<T>
where
	AssetId: Into<<T as orml_tokens::Config>::CurrencyId>,
	Balance: From<<T as orml_tokens::Config>::Balance>,
	Balance: From<<T as pallet_balances::Config>::Balance>,
{
	fn get_by_lock(lock_id: LockIdentifier, currency_id: AssetId, who: T::AccountId) -> Balance {
		if currency_id == NativeAssetId::get() {
			match pallet_balances::Pallet::<T>::locks(who)
				.into_iter()
				.find(|lock| lock.id == lock_id)
			{
				Some(lock) => lock.amount.into(),
				None => Zero::zero(),
			}
		} else {
			match orml_tokens::Pallet::<T>::locks(who, currency_id.into())
				.into_iter()
				.find(|lock| lock.id == lock_id)
			{
				Some(lock) => lock.amount.into(),
				None => Zero::zero(),
			}
		}
	}
}

parameter_types! {
	pub LBPExchangeFee: (u32, u32) = (2, 1_000);
}

impl pallet_lbp::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type LockedBalance = MultiCurrencyLockedBalance<Runtime>;
	type CreatePoolOrigin = SuperMajorityTechCommitteeOrRoot;
	type LBPWeightFunction = pallet_lbp::LBPWeightFunction;
	type AssetPairAccountId = AssetPairAccountId<Self>;
	type WeightInfo = weights::pallet_lbp::BasiliskWeight<Runtime>;
	type MinTradingLimit = MinTradingLimit;
	type MinPoolLiquidity = MinPoolLiquidity;
	type MaxInRatio = MaxInRatio;
	type MaxOutRatio = MaxOutRatio;
	type BlockNumberProvider = RelayChainBlockNumberProvider<Runtime>;
}

#[cfg(feature = "runtime-benchmarks")]
use codec::Decode;
use frame_support::traits::Everything;

pub struct RootAsVestingPallet;
impl EnsureOrigin<RuntimeOrigin> for RootAsVestingPallet {
	type Success = AccountId;

	fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
		Into::<Result<RawOrigin<AccountId>, RuntimeOrigin>>::into(o).and_then(|o| match o {
			RawOrigin::Root => Ok(VestingPalletId::get().into_account_truncating()),
			r => Err(RuntimeOrigin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
		let zero_account_id = AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
			.expect("infinite length input; no invalid inputs for type; qed");
		Ok(RuntimeOrigin::from(RawOrigin::Signed(zero_account_id)))
	}
}

parameter_types! {
	pub MinVestedTransfer: Balance = 100_000;
	pub const MaxVestingSchedules: u32 = 15;
	pub const VestingPalletId: PalletId = PalletId(*b"py/vstng");
}

impl orml_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MinVestedTransfer = MinVestedTransfer;
	type VestedTransferOrigin = RootAsVestingPallet;
	type WeightInfo = weights::orml_vesting::BasiliskWeight<Runtime>;
	type MaxVestingSchedules = MaxVestingSchedules;
	type BlockNumberProvider = RelayChainBlockNumberProvider<Runtime>;
}

parameter_types! {
	pub const MinimumOfferAmount: Balance = UNITS / 100;
	pub const RoyaltyBondAmount: Balance = 0;
}

impl pallet_marketplace::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = KusamaCurrency;
	type WeightInfo = weights::pallet_marketplace::BasiliskWeight<Runtime>;
	type MinimumOfferAmount = MinimumOfferAmount;
	type RoyaltyBondAmount = RoyaltyBondAmount;
}

pub mod ksm {
	use primitives::Balance;

	pub const UNITS: Balance = 1_000_000_000_000;
	pub const CENTS: Balance = UNITS / 30_000;
	pub const MILLICENTS: Balance = CENTS / 1_000;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		(items as Balance * 2_000 * CENTS + (bytes as Balance) * 100 * MILLICENTS) / 10
	}
}

parameter_types! {
	pub const CollectionDeposit: Balance = 0;
	pub const ItemDeposit: Balance = 0;
	pub const KeyLimit: u32 = 256;	// Max 256 bytes per key
	pub const ValueLimit: u32 = 1024;	// Max 1024 bytes per value
	pub const UniquesMetadataDepositBase: Balance = ksm::deposit(1,129);
	pub const AttributeDepositBase: Balance = ksm::deposit(1,0);
	pub const DepositPerByte: Balance = ksm::deposit(0,1);
}

impl pallet_uniques::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = CollectionId;
	type ItemId = ItemId;
	type Currency = KusamaCurrency;
	type ForceOrigin = SuperMajorityCouncilOrRoot;
	// Standard collection creation is disallowed
	type CreateOrigin = AsEnsureOriginWithArg<NeverEnsureOrigin<AccountId>>;
	type Locker = ();
	type CollectionDeposit = CollectionDeposit;
	type ItemDeposit = ItemDeposit;
	type MetadataDepositBase = UniquesMetadataDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = DepositPerByte;
	type StringLimit = primitives::UniquesStringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
	type WeightInfo = ();
}

parameter_types! {
	pub const LMPalletId: PalletId = PalletId(*b"LiqMinId");
	pub const LiquidityMiningNftCollectionId: primitives::CollectionId = 1;
}

impl pallet_xyk_liquidity_mining::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type AMM = XYK;
	type CreateOrigin = UnanimousTechCommitteeOrRoot;
	type PalletId = LMPalletId;
	type NftCollectionId = LiquidityMiningNftCollectionId;
	type NFTHandler = NFT;
	type LiquidityMiningHandler = XYKWarehouseLM;
	type NonDustableWhitelistHandler = Duster;
	type WeightInfo = weights::pallet_xyk_liquidity_mining::BasiliskWeight<Runtime>;
}

parameter_types! {
	pub const WarehouseLMPalletId: PalletId = PalletId(*b"WhouseLm");
	pub const MaxEntriesPerDeposit: u8 = 5; //NOTE: Rebenchmark when this change, TODO:
	pub const MaxYieldFarmsPerGlobalFarm: u8 = 50; //NOTE: Includes deleted/destroyed farms, TODO:
	pub const MinPlannedYieldingPeriods: BlockNumber = 100_800;  //1w, TODO:
	pub const MinTotalFarmRewards: Balance = NATIVE_EXISTENTIAL_DEPOSIT * 100; //TODO:
}

type XYKLiquidityMiningInstance = warehouse_liquidity_mining::Instance1;
impl warehouse_liquidity_mining::Config<XYKLiquidityMiningInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AssetId = AssetId;
	type MultiCurrency = Currencies;
	type PalletId = WarehouseLMPalletId;
	type MinTotalFarmRewards = MinTotalFarmRewards;
	type MinPlannedYieldingPeriods = MinPlannedYieldingPeriods;
	type BlockNumberProvider = RelayChainBlockNumberProvider<Runtime>;
	type AmmPoolId = AccountId;
	type MaxFarmEntriesPerDeposit = MaxEntriesPerDeposit;
	type MaxYieldFarmsPerGlobalFarm = MaxYieldFarmsPerGlobalFarm;
	type AssetRegistry = AssetRegistry;
	type NonDustableWhitelistHandler = Duster;
	type PriceAdjustment = warehouse_liquidity_mining::DefaultPriceAdjustment;
}

// Provides weight info for the router. Router extrinsics can be executed with different AMMs, so we split the router weights into two parts:
// the router extrinsic overhead and the AMM weight.
pub struct RouterWeightInfo;
// Calculates the overhead of Router extrinsics. To do that, we benchmark Router::sell with single LBP trade and subtract the weight of LBP::sell.
// This allows us to calculate the weight of any route by adding the weight of AMM trades to the overhead of a router extrinsic.
impl RouterWeightInfo {
	pub fn sell_and_calculate_sell_trade_amounts_overhead_weight(
		num_of_calc_sell: u32,
		num_of_execute_sell: u32,
	) -> Weight {
		weights::pallet_route_executor::BasiliskWeight::<Runtime>::calculate_and_execute_sell_in_lbp(num_of_calc_sell)
			.saturating_sub(weights::pallet_lbp::BasiliskWeight::<Runtime>::router_execution_sell(
				num_of_calc_sell.saturating_add(num_of_execute_sell),
				num_of_execute_sell,
			))
	}

	pub fn buy_and_calculate_buy_trade_amounts_overhead_weight(
		num_of_calc_buy: u32,
		num_of_execute_buy: u32,
	) -> Weight {
		let router_weight = weights::pallet_route_executor::BasiliskWeight::<Runtime>::calculate_and_execute_buy_in_lbp(
			num_of_calc_buy,
			num_of_execute_buy,
		);
		// Handle this case separately. router_execution_buy provides incorrect weight for the case when only calculate_buy is executed.
		let lbp_weight = if (num_of_calc_buy, num_of_execute_buy) == (1, 0) {
			weights::pallet_lbp::BasiliskWeight::<Runtime>::calculate_buy()
		} else {
			weights::pallet_lbp::BasiliskWeight::<Runtime>::router_execution_buy(
				num_of_calc_buy.saturating_add(num_of_execute_buy),
				num_of_execute_buy,
			)
		};
		router_weight.saturating_sub(lbp_weight)
	}

	pub fn set_route_overweight() -> Weight {
		let number_of_times_calculate_sell_amounts_executed = 5; //4 calculations + in the validation
		let number_of_times_execute_sell_amounts_executed = 0; //We do have it once executed in the validation of the route, but it is without writing to database (as rolled back), and since we pay back successful set_route, we just keep this overhead

		let set_route_overweight = weights::pallet_route_executor::BasiliskWeight::<Runtime>::set_route_for_xyk();

		set_route_overweight.saturating_sub(weights::pallet_xyk::BasiliskWeight::<Runtime>::router_execution_sell(
			number_of_times_calculate_sell_amounts_executed,
			number_of_times_execute_sell_amounts_executed,
		))
	}
}

impl AmmTradeWeights<Trade<AssetId>> for RouterWeightInfo {
	// Used in Router::sell extrinsic, which calls AMM::calculate_sell and AMM::execute_sell
	fn sell_weight(route: &[Trade<AssetId>]) -> Weight {
		let mut weight = Weight::zero();
		let c = 1; // number of times AMM::calculate_sell is executed
		let e = 1; // number of times AMM::execute_sell is executed

		for trade in route {
			weight.saturating_accrue(Self::sell_and_calculate_sell_trade_amounts_overhead_weight(0, 1));

			let lbp_weight = weights::pallet_lbp::BasiliskWeight::<Runtime>::router_execution_sell(c, e);
			let xyk_weight = weights::pallet_xyk::BasiliskWeight::<Runtime>::router_execution_sell(c, e)
				.saturating_add(<Runtime as pallet_xyk::Config>::AMMHandler::on_trade_weight());

			let amm_weight = match trade.pool {
				PoolType::LBP => lbp_weight,
				PoolType::XYK => xyk_weight,
				_ => lbp_weight.max(xyk_weight),
			};
			weight.saturating_accrue(amm_weight);
		}

		weight
	}

	// Used in Router::buy extrinsic, which calls AMM::calculate_buy and AMM::execute_buy
	fn buy_weight(route: &[Trade<AssetId>]) -> Weight {
		let mut weight = Weight::zero();
		let c = 1; // number of times AMM::calculate_buy is executed
		let e = 1; // number of times AMM::execute_buy is executed

		for trade in route {
			weight.saturating_accrue(Self::buy_and_calculate_buy_trade_amounts_overhead_weight(0, 1));

			let lbp_weight = weights::pallet_lbp::BasiliskWeight::<Runtime>::router_execution_buy(c, e);
			let xyk_weight = weights::pallet_xyk::BasiliskWeight::<Runtime>::router_execution_buy(c, e)
				.saturating_add(<Runtime as pallet_xyk::Config>::AMMHandler::on_trade_weight());

			let amm_weight = match trade.pool {
				PoolType::LBP => lbp_weight,
				PoolType::XYK => xyk_weight,
				_ => lbp_weight.max(xyk_weight),
			};
			weight.saturating_accrue(amm_weight);
		}

		weight
	}

	// Used in DCA::schedule extrinsic, which calls Router::calculate_buy_trade_amounts
	fn calculate_buy_trade_amounts_weight(route: &[Trade<AssetId>]) -> Weight {
		let mut weight = Weight::zero();
		let c = 1; // number of times AMM::calculate_buy is executed
		let e = 0; // number of times AMM::execute_buy is executed

		for trade in route {
			weight.saturating_accrue(Self::buy_and_calculate_buy_trade_amounts_overhead_weight(1, 0));

			let lbp_weight = weights::pallet_lbp::BasiliskWeight::<Runtime>::router_execution_buy(c, e);
			let xyk_weight = weights::pallet_xyk::BasiliskWeight::<Runtime>::router_execution_buy(c, e)
				.saturating_add(<Runtime as pallet_xyk::Config>::AMMHandler::on_trade_weight());

			let amm_weight = match trade.pool {
				PoolType::LBP => lbp_weight,
				PoolType::XYK => xyk_weight,
				_ => lbp_weight.max(xyk_weight),
			};
			weight.saturating_accrue(amm_weight);
		}

		weight
	}

	// Used in DCA::on_initialize for Order::Sell, which calls Router::calculate_sell_trade_amounts and Router::sell.
	fn sell_and_calculate_sell_trade_amounts_weight(route: &[Trade<AssetId>]) -> Weight {
		let mut weight = Weight::zero();
		let c = 2; // number of times AMM::calculate_sell is executed
		let e = 1; // number of times AMM::execute_sell is executed

		for trade in route {
			weight.saturating_accrue(Self::sell_and_calculate_sell_trade_amounts_overhead_weight(1, 1));

			let lbp_weight = weights::pallet_lbp::BasiliskWeight::<Runtime>::router_execution_sell(c, e);
			let xyk_weight = weights::pallet_xyk::BasiliskWeight::<Runtime>::router_execution_sell(c, e)
				.saturating_add(<Runtime as pallet_xyk::Config>::AMMHandler::on_trade_weight());

			let amm_weight = match trade.pool {
				PoolType::LBP => lbp_weight,
				PoolType::XYK => xyk_weight,
				_ => lbp_weight.max(xyk_weight),
			};
			weight.saturating_accrue(amm_weight);
		}

		weight
	}

	// Used in DCA::on_initialize for Order::Buy, which calls 2 * Router::calculate_buy_trade_amounts and Router::buy.
	fn buy_and_calculate_buy_trade_amounts_weight(route: &[Trade<AssetId>]) -> Weight {
		let mut weight = Weight::zero();
		let c = 3; // number of times AMM::calculate_buy is executed
		let e = 1; // number of times AMM::execute_buy is executed

		for trade in route {
			weight.saturating_accrue(Self::buy_and_calculate_buy_trade_amounts_overhead_weight(2, 1));

			let lbp_weight = weights::pallet_lbp::BasiliskWeight::<Runtime>::router_execution_buy(c, e);
			let xyk_weight = weights::pallet_xyk::BasiliskWeight::<Runtime>::router_execution_buy(c, e)
				.saturating_add(<Runtime as pallet_xyk::Config>::AMMHandler::on_trade_weight());

			let amm_weight = match trade.pool {
				PoolType::LBP => lbp_weight,
				PoolType::XYK => xyk_weight,
				_ => lbp_weight.max(xyk_weight),
			};
			weight.saturating_accrue(amm_weight);
		}

		weight
	}

	fn set_route_weight(route: &[Trade<AssetId>]) -> Weight {
		let mut weight = Weight::zero();

		//We ignore the calls for AMM:get_liquidty_depth, as the same logic happens in AMM calculation/execution

		//Overweight
		weight.saturating_accrue(Self::set_route_overweight());

		//Add a sell weight as we do a dry-run sell as validation
		weight.saturating_accrue(Self::sell_weight(route));

		//For the stored route we expect a worst case with max number of trades in the most expensive pool which is LBP
		//We have have two sell calculation for that, normal and inverse
		weights::pallet_lbp::BasiliskWeight::<Runtime>::router_execution_sell(2, 0)
			.checked_mul(pallet_route_executor::MAX_NUMBER_OF_TRADES.into());

		let lbp_weight = weights::pallet_lbp::BasiliskWeight::<Runtime>::router_execution_sell(1, 0);
		let xyk_weight = weights::pallet_xyk::BasiliskWeight::<Runtime>::router_execution_sell(1, 0);

		//Calculate sell amounts for the new route
		for trade in route {
			let amm_weight = match trade.pool {
				PoolType::LBP => lbp_weight,
				PoolType::XYK => xyk_weight,
				_ => lbp_weight.max(xyk_weight),
			};
			weight.saturating_accrue(amm_weight);
		}

		//Calculate sell amounts for the inversed new route
		for trade in inverse_route(route.to_vec()) {
			let amm_weight = match trade.pool {
				PoolType::LBP => lbp_weight,
				PoolType::XYK => xyk_weight,
				_ => lbp_weight.max(xyk_weight),
			};
			weight.saturating_accrue(amm_weight);
		}

		weight
	}

	fn force_insert_route_weight() -> Weight {
		//Since we don't have any AMM specific thing in the extrinsic, we just return the plain weight
		weights::pallet_route_executor::BasiliskWeight::<Runtime>::force_insert_route()
	}
}

parameter_types! {
	pub DefaultRoutePoolType: PoolType<AssetId> = PoolType::XYK;
}
impl pallet_route_executor::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AssetId = AssetId;
	type Balance = Balance;
	type Currency = FungibleCurrencies<Runtime>;
	type AMM = (XYK, LBP);
	type NativeAssetId = NativeAssetId;
	type DefaultRoutePoolType = DefaultRoutePoolType;
	type WeightInfo = RouterWeightInfo;
	type InspectRegistry = AssetRegistry;
	type TechnicalOrigin = SuperMajorityTechCommitteeOrRoot;
}

parameter_types! {
	pub SupportedPeriods: BoundedVec<OraclePeriod, ConstU32<{ pallet_ema_oracle::MAX_PERIODS }>> = BoundedVec::truncate_from(
		vec![OraclePeriod::LastBlock, OraclePeriod::Short, OraclePeriod::Hour, OraclePeriod::Day, OraclePeriod::Week]
	);
	// There are currently only a few pools, so the number of entries per block is limited.
	// NOTE: Needs to be updated once the number of pools grows.
	pub MaxUniqueOracleEntries: u32 = 30;
}

impl pallet_ema_oracle::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_ema_oracle::BasiliskWeight<Runtime>;
	type AuthorityOrigin = SuperMajorityTechCommitteeOrRoot;
	type BlockNumberProvider = RelayChainBlockNumberProvider<Runtime>;
	type SupportedPeriods = SupportedPeriods;
	type OracleWhitelist = Everything;
	type MaxUniqueEntries = MaxUniqueOracleEntries;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = (); //TODO: implement helper
}

parameter_types! {
	pub ReserveCollectionIdUpTo: u128 = 999_999;
}

impl pallet_nft::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_nft::BasiliskWeight<Runtime>;
	type NftCollectionId = CollectionId;
	type NftItemId = ItemId;
	type CollectionType = pallet_nft::CollectionType;
	type Permissions = pallet_nft::NftPermissions;
	type ReserveCollectionIdUpTo = ReserveCollectionIdUpTo;
}
