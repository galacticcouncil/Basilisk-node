use crate::{Config, Error, Pallet};
use frame_support::ensure;
use hydradx_traits::router::{ExecutorError, PoolType, TradeExecution};
use hydradx_traits::AMM;
use orml_traits::MultiCurrency;
use primitives::asset::AssetPair;
use primitives::{AssetId, Balance};
use sp_runtime::traits::Zero;
use sp_runtime::DispatchError;
use frame_support::traits::Get;

impl<T: Config> TradeExecution<T::AccountId, AssetId, Balance> for Pallet<T> {
	type Error = DispatchError;

	fn calculate_sell(
		pool_type: PoolType<AssetId>,
		asset_in: AssetId,
		asset_out: AssetId,
		amount_in: Balance,
	) -> Result<Balance, ExecutorError<Self::Error>> {
		if pool_type != PoolType::XYK {
			return Err(ExecutorError::NotSupported);
		}

		let assets = AssetPair { asset_in, asset_out };

		if !Self::exists(assets) {
			return Err(ExecutorError::Error(Error::<T>::TokenPoolNotFound.into()));
		}

		let pair_account = Self::get_pair_id(assets);

		let asset_in_reserve = T::Currency::free_balance(assets.asset_in, &pair_account);
		let asset_out_reserve = T::Currency::free_balance(assets.asset_out, &pair_account);

		let amount_out =
			hydra_dx_math::xyk::calculate_out_given_in(asset_in_reserve, asset_out_reserve, amount_in)
				.map_err(|_| ExecutorError::Error(Error::<T>::SellAssetAmountInvalid.into()))?;

		let transfer_fee = Self::calculate_fee(amount_out).map_err(ExecutorError::Error)?;

		let amount_out_without_fee = amount_out
			.checked_sub(transfer_fee)
			.ok_or(ExecutorError::Error(Error::<T>::SellAssetAmountInvalid.into()))?;

		//ensure!(asset_out_reserve > amount_out, Error::<T>::InsufficientAssetBalance);

		Ok(amount_out_without_fee)
	}

	fn calculate_buy(
		pool_type: PoolType<AssetId>,
		asset_in: AssetId,
		asset_out: AssetId,
		amount_out: Balance,
	) -> Result<Balance, ExecutorError<Self::Error>> {
		if pool_type != PoolType::XYK {
			return Err(ExecutorError::NotSupported);
		}

		let assets = AssetPair { asset_in, asset_out };

		ensure!(
			Self::exists(assets),
			ExecutorError::Error(Error::<T>::TokenPoolNotFound.into())
		);

		let pair_account = Self::get_pair_id(assets);

		let asset_out_reserve = T::Currency::free_balance(assets.asset_out, &pair_account);
		let asset_in_reserve = T::Currency::free_balance(assets.asset_in, &pair_account);

		ensure!(
			asset_out_reserve > amount_out,
			ExecutorError::Error(Error::<T>::InsufficientPoolAssetBalance.into())
		);

		ensure!(
			amount_out >= T::MinTradingLimit::get(),
			ExecutorError::Error(Error::<T>::InsufficientTradingAmount.into())
		);

		let amount_in =
			hydra_dx_math::xyk::calculate_in_given_out(asset_out_reserve, asset_in_reserve, amount_out)
				.map_err(|_| ExecutorError::Error(Error::<T>::BuyAssetAmountInvalid.into()))?;

		let transfer_fee = Self::calculate_fee(amount_in).map_err(ExecutorError::Error)?;

		let amount_in_with_fee = amount_in
			.checked_add(transfer_fee)
			.ok_or(ExecutorError::Error(Error::<T>::BuyAssetAmountInvalid.into()))?;

		Ok(amount_in_with_fee)
	}

	fn execute_sell(
		pool_type: PoolType<AssetId>,
		who: &T::AccountId,
		asset_in: AssetId,
		asset_out: AssetId,
		amount_in: Balance,
	) -> Result<(), ExecutorError<Self::Error>> {
		if pool_type != PoolType::XYK {
			return Err(ExecutorError::NotSupported);
		}

		<Self as AMM<T::AccountId, AssetId, AssetPair, Balance>>::sell(
			who,
			AssetPair { asset_in, asset_out },
			amount_in,
			Balance::zero(),
			false,
		)
		.map_err(|v| ExecutorError::Error(v))
	}

	fn execute_buy(
		pool_type: PoolType<AssetId>,
		who: &T::AccountId,
		asset_in: AssetId,
		asset_out: AssetId,
		amount_out: Balance,
	) -> Result<(), ExecutorError<Self::Error>> {
		if pool_type != PoolType::XYK {
			return Err(ExecutorError::NotSupported);
		}

		<Self as AMM<T::AccountId, AssetId, AssetPair, Balance>>::buy(
			who,
			AssetPair { asset_in, asset_out },
			amount_out,
			Balance::MAX,
			false,
		)
		.map_err(|v| ExecutorError::Error(v))
	}
}
