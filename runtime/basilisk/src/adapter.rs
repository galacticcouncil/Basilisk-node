use basilisk_traits::{
	oracle::{AggregatedPriceOracle, OraclePeriod, PriceOracle},
	router::{PoolType, Trade},
};
use frame_support::sp_runtime::DispatchResult;
use frame_support::traits::{BalanceStatus, ExistenceRequirement};
use frame_system::pallet_prelude::BlockNumberFor;
use hydra_dx_math::{
	ema::EmaPrice,
	support::rational::{round_u512_to_rational, Rounding},
};
use orml_traits::currency::TransferAll;
use orml_traits::{
	LockIdentifier, MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency, MultiReservableCurrency,
	NamedMultiReservableCurrency,
};
use pallet_ema_oracle::OracleError;
use primitive_types::U512;
use primitives::BlockNumber;
use sp_runtime::DispatchError;
use sp_std::{marker::PhantomData, vec::Vec};

pub struct OrmlTokensAdapter<T>(PhantomData<T>);

impl<T: orml_tokens::Config + frame_system::Config> MultiCurrency<T::AccountId> for OrmlTokensAdapter<T> {
	type CurrencyId = <T as orml_tokens::Config>::CurrencyId;
	type Balance = <T as orml_tokens::Config>::Balance;

	fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::minimum_balance(currency_id)
	}

	fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::total_issuance(currency_id)
	}

	fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::total_balance(currency_id, who)
	}

	fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::free_balance(currency_id, who)
	}

	fn ensure_can_withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::ensure_can_withdraw(currency_id, who, amount)
	}

	fn transfer(
		currency_id: Self::CurrencyId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: Self::Balance,
		existence_requirement: ExistenceRequirement,
	) -> DispatchResult {
		let res = <orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::transfer(
			currency_id,
			from,
			to,
			amount,
			existence_requirement,
		);

		if res.is_ok() {
			<frame_system::Pallet<T>>::deposit_event(
				<T as orml_tokens::Config>::RuntimeEvent::from(orml_tokens::Event::Transfer {
					currency_id,
					from: from.clone(),
					to: to.clone(),
					amount,
				})
				.into(),
			);
		}

		res
	}

	fn deposit(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::deposit(currency_id, who, amount)
	}

	fn withdraw(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
		existence_requirement: ExistenceRequirement,
	) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::withdraw(
			currency_id,
			who,
			amount,
			existence_requirement,
		)
	}

	fn can_slash(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> bool {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::can_slash(currency_id, who, value)
	}

	fn slash(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiCurrency<T::AccountId>>::slash(currency_id, who, amount)
	}
}

impl<T: orml_tokens::Config + frame_system::Config> MultiCurrencyExtended<T::AccountId> for OrmlTokensAdapter<T> {
	type Amount = <T as orml_tokens::Config>::Amount;

	fn update_balance(currency_id: Self::CurrencyId, who: &T::AccountId, by_amount: Self::Amount) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiCurrencyExtended<T::AccountId>>::update_balance(currency_id, who, by_amount)
	}
}

impl<T: orml_tokens::Config + frame_system::Config> MultiReservableCurrency<T::AccountId> for OrmlTokensAdapter<T> {
	fn can_reserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> bool {
		<orml_tokens::Pallet<T> as MultiReservableCurrency<T::AccountId>>::can_reserve(currency_id, who, value)
	}

	fn slash_reserved(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiReservableCurrency<T::AccountId>>::slash_reserved(currency_id, who, value)
	}

	fn reserved_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiReservableCurrency<T::AccountId>>::reserved_balance(currency_id, who)
	}

	fn reserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiReservableCurrency<T::AccountId>>::reserve(currency_id, who, value)
	}

	fn unreserve(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		<orml_tokens::Pallet<T> as MultiReservableCurrency<T::AccountId>>::unreserve(currency_id, who, value)
	}

	fn repatriate_reserved(
		currency_id: Self::CurrencyId,
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> Result<Self::Balance, DispatchError> {
		<orml_tokens::Pallet<T> as MultiReservableCurrency<T::AccountId>>::repatriate_reserved(
			currency_id,
			slashed,
			beneficiary,
			value,
			status,
		)
	}
}

impl<T: orml_tokens::Config + frame_system::Config> MultiLockableCurrency<T::AccountId> for OrmlTokensAdapter<T> {
	type Moment = BlockNumberFor<T>;

	fn set_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiLockableCurrency<T::AccountId>>::set_lock(lock_id, currency_id, who, amount)
	}

	fn extend_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiLockableCurrency<T::AccountId>>::extend_lock(lock_id, currency_id, who, amount)
	}

	fn remove_lock(lock_id: LockIdentifier, currency_id: Self::CurrencyId, who: &T::AccountId) -> DispatchResult {
		<orml_tokens::Pallet<T> as MultiLockableCurrency<T::AccountId>>::remove_lock(lock_id, currency_id, who)
	}
}

impl<T: orml_tokens::Config> TransferAll<T::AccountId> for OrmlTokensAdapter<T> {
	fn transfer_all(source: &T::AccountId, dest: &T::AccountId) -> DispatchResult {
		<orml_tokens::Pallet<T> as TransferAll<T::AccountId>>::transfer_all(source, dest)
	}
}

impl<T: orml_tokens::Config> NamedMultiReservableCurrency<T::AccountId> for OrmlTokensAdapter<T> {
	type ReserveIdentifier = T::ReserveIdentifier;

	fn slash_reserved_named(
		id: &Self::ReserveIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		value: Self::Balance,
	) -> Self::Balance {
		<orml_tokens::Pallet<T> as NamedMultiReservableCurrency<T::AccountId>>::slash_reserved_named(
			id,
			currency_id,
			who,
			value,
		)
	}

	fn reserved_balance_named(
		id: &Self::ReserveIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
	) -> Self::Balance {
		<orml_tokens::Pallet<T> as NamedMultiReservableCurrency<T::AccountId>>::reserved_balance_named(
			id,
			currency_id,
			who,
		)
	}

	fn reserve_named(
		id: &Self::ReserveIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		value: Self::Balance,
	) -> DispatchResult {
		<orml_tokens::Pallet<T> as NamedMultiReservableCurrency<T::AccountId>>::reserve_named(
			id,
			currency_id,
			who,
			value,
		)
	}

	fn unreserve_named(
		id: &Self::ReserveIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		value: Self::Balance,
	) -> Self::Balance {
		<orml_tokens::Pallet<T> as NamedMultiReservableCurrency<T::AccountId>>::unreserve_named(
			id,
			currency_id,
			who,
			value,
		)
	}

	fn repatriate_reserved_named(
		id: &Self::ReserveIdentifier,
		currency_id: Self::CurrencyId,
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> Result<Self::Balance, DispatchError> {
		<orml_tokens::Pallet<T> as NamedMultiReservableCurrency<T::AccountId>>::repatriate_reserved_named(
			id,
			currency_id,
			slashed,
			beneficiary,
			value,
			status,
		)
	}

	fn ensure_reserved_named(
		id: &Self::ReserveIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		value: Self::Balance,
	) -> DispatchResult {
		<orml_tokens::Pallet<T> as NamedMultiReservableCurrency<T::AccountId>>::ensure_reserved_named(
			id,
			currency_id,
			who,
			value,
		)
	}

	fn unreserve_all_named(
		id: &Self::ReserveIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
	) -> Self::Balance {
		<orml_tokens::Pallet<T> as NamedMultiReservableCurrency<T::AccountId>>::unreserve_all_named(
			id,
			currency_id,
			who,
		)
	}

	fn slash_all_reserved_named(
		id: &Self::ReserveIdentifier,
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
	) -> Self::Balance {
		<orml_tokens::Pallet<T> as NamedMultiReservableCurrency<T::AccountId>>::slash_all_reserved_named(
			id,
			currency_id,
			who,
		)
	}

	fn repatriate_all_reserved_named(
		id: &Self::ReserveIdentifier,
		currency_id: Self::CurrencyId,
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		status: BalanceStatus,
	) -> DispatchResult {
		<orml_tokens::Pallet<T> as NamedMultiReservableCurrency<T::AccountId>>::repatriate_all_reserved_named(
			id,
			currency_id,
			slashed,
			beneficiary,
			status,
		)
	}
}

pub struct OraclePriceProvider<AssetId, AggregatedPriceGetter>(PhantomData<(AssetId, AggregatedPriceGetter)>);

impl<AssetId, AggregatedPriceGetter> PriceOracle<AssetId> for OraclePriceProvider<AssetId, AggregatedPriceGetter>
where
	u32: From<AssetId>,
	AggregatedPriceGetter: AggregatedPriceOracle<AssetId, BlockNumber, EmaPrice, Error = OracleError>,
	AssetId: Clone + Copy,
{
	type Price = EmaPrice;

	/// We calculate prices for trade (in a route) then making the product of them
	fn price(route: &[Trade<AssetId>], period: OraclePeriod) -> Option<EmaPrice> {
		let mut prices: Vec<EmaPrice> = Vec::with_capacity(route.len());
		for trade in route {
			let asset_a = trade.asset_in;
			let asset_b = trade.asset_out;
			let price = match trade.pool {
				PoolType::XYK => {
					let price_result = AggregatedPriceGetter::get_price(
						asset_a,
						asset_b,
						period,
						crate::XYKOracleSourceIdentifier::get(),
					);

					match price_result {
						Ok(price) => price.0,
						Err(OracleError::SameAsset) => EmaPrice::from(1),
						Err(_) => return None,
					}
				}
				_ => return None,
			};

			prices.push(price);
		}

		if prices.is_empty() {
			return None;
		}

		let nominator = prices
			.iter()
			.try_fold(U512::from(1u128), |acc, price| acc.checked_mul(U512::from(price.n)))?;

		let denominator = prices
			.iter()
			.try_fold(U512::from(1u128), |acc, price| acc.checked_mul(U512::from(price.d)))?;

		let rat_as_u128 = round_u512_to_rational((nominator, denominator), Rounding::Nearest);

		Some(EmaPrice::new(rat_as_u128.0, rat_as_u128.1))
	}
}
