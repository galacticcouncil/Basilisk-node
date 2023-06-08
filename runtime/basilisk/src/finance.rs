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
use crate::democracy::{SuperMajorityCouncilOrRoot, SuperMajorityTechCommitteeOrRoot, UnanimousTechCommitteeOrRoot};
use crate::system::NativeAssetId;
use adapter::OrmlTokensAdapter;

use hydradx_adapters::inspect::MultiInspectAdapter;
use hydradx_traits::{AssetPairAccountIdFor, LockedBalance, OraclePeriod};
use pallet_currencies::BasicCurrencyAdapter;
use pallet_transaction_multi_payment::{AddTxAssetOnAccount, RemoveTxAssetOnKilled};
use primitives::constants::{
	chain::{DISCOUNTED_FEE, MAX_IN_RATIO, MAX_OUT_RATIO, MIN_POOL_LIQUIDITY, MIN_TRADING_LIMIT},
	currency::{NATIVE_EXISTENTIAL_DEPOSIT, UNITS},
};

use codec::Decode;
use frame_support::{
	parameter_types,
	sp_runtime::{app_crypto::sp_core::crypto::UncheckedFrom, traits::Zero},
	traits::{AsEnsureOriginWithArg, Contains, Defensive, EnsureOrigin, NeverEnsureOrigin, Get, LockIdentifier},
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

// pallet balances
parameter_types! {
	pub const NativeExistentialDeposit: u128 = NATIVE_EXISTENTIAL_DEPOSIT;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = Treasury;
	type ExistentialDeposit = NativeExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = weights::balances::BasiliskWeight<Runtime>;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ();
}

// pallet orml tokens
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
	type WeightInfo = weights::tokens::BasiliskWeight<Runtime>;
	type ExistentialDeposits = AssetRegistry;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = ();
	type DustRemovalWhitelist = DustRemovalWhitelist;
	type CurrencyHooks = CurrencyHooks;
}

// The latest versions of the orml-currencies pallet don't emit events.
// The infrastructure relies on the events from this pallet, so we use the latest version of
// the pallet that contains and emit events and was updated to the polkadot version we use.
impl pallet_currencies::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = OrmlTokensAdapter<Runtime>;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type GetNativeCurrencyId = NativeAssetId;
	type WeightInfo = weights::currencies::BasiliskWeight<Runtime>;
}

// pallet xyk
parameter_types! {
	pub ExchangeFee: (u32, u32) = (3, 1_000);
	pub const MinTradingLimit: Balance = MIN_TRADING_LIMIT;
	pub const MinPoolLiquidity: Balance = MIN_POOL_LIQUIDITY;
	pub const MaxInRatio: u128 = MAX_IN_RATIO;
	pub const MaxOutRatio: u128 = MAX_OUT_RATIO;
	pub const RegistryStrLimit: u32 = 32;
	pub const DiscountedFee: (u32, u32) = DISCOUNTED_FEE;
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

impl pallet_xyk::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AssetRegistry = AssetRegistry;
	type AssetPairAccountId = AssetPairAccountId<Self>;
	type Currency = Currencies;
	type NativeAssetId = NativeAssetId;
	type WeightInfo = weights::xyk::BasiliskWeight<Runtime>;
	type GetExchangeFee = ExchangeFee;
	type MinTradingLimit = MinTradingLimit;
	type MinPoolLiquidity = MinPoolLiquidity;
	type MaxInRatio = MaxInRatio;
	type MaxOutRatio = MaxOutRatio;
	type CanCreatePool = pallet_lbp::DisallowWhenLBPPoolRunning<Runtime>;
	type AMMHandler = pallet_ema_oracle::OnActivityHandler<Runtime>;
	type DiscountedFee = DiscountedFee;
	type NonDustableWhitelistHandler = Duster;
}

// pallet lbp
parameter_types! {
	pub LBPExchangeFee: (u32, u32) = (2, 1_000);
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

impl pallet_lbp::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type LockedBalance = MultiCurrencyLockedBalance<Runtime>;
	type CreatePoolOrigin = SuperMajorityTechCommitteeOrRoot;
	type LBPWeightFunction = pallet_lbp::LBPWeightFunction;
	type AssetPairAccountId = AssetPairAccountId<Self>;
	type MinTradingLimit = MinTradingLimit;
	type MinPoolLiquidity = MinPoolLiquidity;
	type MaxInRatio = MaxInRatio;
	type MaxOutRatio = MaxOutRatio;
	type WeightInfo = weights::lbp::BasiliskWeight<Runtime>;
	type BlockNumberProvider = RelayChainBlockNumberProvider<Runtime>;
}

// pallet vesting
parameter_types! {
	pub MinVestedTransfer: Balance = 100_000;
	pub const MaxVestingSchedules: u32 = 15;
	pub const VestingPalletId: PalletId = PalletId(*b"py/vstng");
}

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

impl orml_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MinVestedTransfer = MinVestedTransfer;
	type VestedTransferOrigin = RootAsVestingPallet;
	type WeightInfo = weights::vesting::BasiliskWeight<Runtime>;
	type MaxVestingSchedules = MaxVestingSchedules;
	type BlockNumberProvider = RelayChainBlockNumberProvider<Runtime>;
}

// pallet marketplace
parameter_types! {
	pub const MinimumOfferAmount: Balance = UNITS / 100;
	pub const RoyaltyBondAmount: Balance = 0;
}

impl pallet_marketplace::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = KusamaCurrency;
	type WeightInfo = pallet_marketplace::weights::BasiliskWeight<Runtime>;
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

// pallet uniques
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
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
}

// pallet liquidity mining
parameter_types! {
	pub const LMPalletId: PalletId = PalletId(*b"LiqMinId");
	pub const LiquidityMiningNftCollectionId: primitives::CollectionId = 1;
}

impl pallet_xyk_liquidity_mining::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type MultiCurrency = Currencies;
	type CreateOrigin = UnanimousTechCommitteeOrRoot;
	type PalletId = LMPalletId;
	type NftCollectionId = LiquidityMiningNftCollectionId;
	type AMM = XYK;
	type WeightInfo = weights::xyk_liquidity_mining::BasiliskWeight<Runtime>;
	type NFTHandler = NFT;
	type LiquidityMiningHandler = XYKWarehouseLM;
	type NonDustableWhitelistHandler = Duster;
}

// warehouse pallet liquidity mining
parameter_types! {
	pub const WarehouseLMPalletId: PalletId = PalletId(*b"WhouseLm");
	pub const MaxEntriesPerDeposit: u8 = 5; //NOTE: Rebenchmark when this change, TODO:
	pub const MaxYieldFarmsPerGlobalFarm: u8 = 50; //NOTE: Includes deleted/destroyed farms, TODO:
	pub const MinPlannedYieldingPeriods: BlockNumber = 100_800;  //1w, TODO:
	pub const MinTotalFarmRewards: Balance = NATIVE_EXISTENTIAL_DEPOSIT * 100; //TODO:
}

type XYKLiquidityMiningInstance = warehouse_liquidity_mining::Instance1;
impl warehouse_liquidity_mining::Config<XYKLiquidityMiningInstance> for Runtime {
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
	type RuntimeEvent = RuntimeEvent;
	type PriceAdjustment = warehouse_liquidity_mining::DefaultPriceAdjustment;
}

// pallet route executor
parameter_types! {
	pub const MaxNumberOfTrades: u8 = 5;
}

impl pallet_route_executor::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AssetId = AssetId;
	type Balance = Balance;
	type MaxNumberOfTrades = MaxNumberOfTrades;
	type Currency = MultiInspectAdapter<AccountId, AssetId, Balance, Balances, Tokens, NativeAssetId>;
	type AMM = (XYK, LBP);
	type WeightInfo = weights::route_executor::BasiliskWeight<Runtime>;
}

// pallet ema oracle
parameter_types! {
	pub SupportedPeriods: BoundedVec<OraclePeriod, ConstU32<{ pallet_ema_oracle::MAX_PERIODS }>> = BoundedVec::truncate_from(
		vec![OraclePeriod::LastBlock, OraclePeriod::Hour, OraclePeriod::Day, OraclePeriod::Week]
	);
	// There are currently only a few pools, so the number of entries per block is limited.
	// NOTE: Needs to be updated once the number of pools grows.
	pub MaxUniqueOracleEntries: u32 = 30;
}

impl pallet_ema_oracle::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::ema_oracle::BasiliskWeight<Runtime>;
	type BlockNumberProvider = RelayChainBlockNumberProvider<Runtime>;
	type SupportedPeriods = SupportedPeriods;
	type MaxUniqueEntries = MaxUniqueOracleEntries;
}

// pallet nft
parameter_types! {
	pub ReserveCollectionIdUpTo: u128 = 999_999;
}

impl pallet_nft::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::nft::BasiliskWeight<Runtime>;
	type NftCollectionId = CollectionId;
	type NftItemId = ItemId;
	type CollectionType = pallet_nft::CollectionType;
	type Permissions = pallet_nft::NftPermissions;
	type ReserveCollectionIdUpTo = ReserveCollectionIdUpTo;
}
