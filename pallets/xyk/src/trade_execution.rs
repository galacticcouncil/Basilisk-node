use hydradx_traits::router::{AmountWithFee, ExecutorError, PoolType, TradeExecution};
use hydradx_traits::AMMTransfer;
use hydradx_traits::AMM;
use orml_traits::MultiCurrency;
use primitives::{asset::AssetPair, AssetId, Balance};
use sp_runtime::DispatchError;

impl<T: crate::Config> TradeExecution<T::AccountId, AssetId, Balance> for crate::Pallet<T> {
	type TradeCalculationResult = AmountWithFee<Balance>;
	type Error = DispatchError;

	fn calculate_sell(
		pool_type: PoolType<AssetId>,
		asset_in: AssetId,
		asset_out: AssetId,
		amount_in: Self::TradeCalculationResult,
	) -> Result<Self::TradeCalculationResult, ExecutorError<Self::Error>> {
		if pool_type != PoolType::XYK {
			return Err(ExecutorError::NotSupported);
		}

		let pair = AssetPair { asset_in, asset_out };

		Self::check_if_pool_exists(pair).map_err(|err| ExecutorError::Error(err))?;

		let pair_account = <crate::Pallet<T>>::get_pair_id(pair);

		let asset_in_reserve = T::Currency::free_balance(asset_in, &pair_account);
		let asset_out_reserve = T::Currency::free_balance(asset_out, &pair_account);

		let amount_in_without_fee =
			amount_in
				.amount
				.checked_sub(amount_in.fee)
				.ok_or(ExecutorError::Error(DispatchError::Other(
					"Error while calculating sell trade",
				)))?;

		let amount_out =
			hydra_dx_math::xyk::calculate_out_given_in(asset_in_reserve, asset_out_reserve, amount_in_without_fee)
				.map_err(|_| ExecutorError::Error(DispatchError::Other("Error while calculating sell trade")))?;

		let transfer_fee = Self::calculate_fee(amount_out).map_err(|de| ExecutorError::Error(de))?;

		Ok(AmountWithFee::new(amount_out, transfer_fee))
	}

	fn calculate_buy(
		pool_type: PoolType<AssetId>,
		asset_in: AssetId,
		asset_out: AssetId,
		amount_out: Self::TradeCalculationResult,
	) -> Result<Self::TradeCalculationResult, ExecutorError<Self::Error>> {
		if pool_type != PoolType::XYK {
			return Err(ExecutorError::NotSupported);
		}

		let pair = AssetPair { asset_in, asset_out };
		Self::check_if_pool_exists(pair).map_err(|err| ExecutorError::Error(err))?;

		let pair_account = <crate::Pallet<T>>::get_pair_id(pair);

		let asset_in_reserve = T::Currency::free_balance(asset_in, &pair_account);
		let asset_out_reserve = T::Currency::free_balance(asset_out, &pair_account);

		let amount_out_with_fee = amount_out
			.amount
			.checked_add(amount_out.fee)
			.ok_or(ExecutorError::Error(DispatchError::Other(
				"Error while calculating buy trade",
			)))?;

		let amount_in =
			hydra_dx_math::xyk::calculate_in_given_out(asset_out_reserve, asset_in_reserve, amount_out_with_fee)
				.map_err(|_| ExecutorError::Error(DispatchError::Other("Error while calculating buy trade")))?;

		let transfer_fee = Self::calculate_fee(amount_in).map_err(|de| ExecutorError::Error(de))?;

		Ok(AmountWithFee::new(amount_in, transfer_fee))
	}

	fn execute_sell(
		pool_type: PoolType<AssetId>,
		who: &T::AccountId,
		asset_in: AssetId,
		asset_out: AssetId,
		amount_in: Self::TradeCalculationResult,
	) -> Result<(), ExecutorError<Self::Error>> {
		if pool_type != PoolType::XYK {
			return Err(ExecutorError::NotSupported);
		}

		let amount_out = Self::calculate_sell(pool_type, asset_in, asset_out, amount_in)?;

		let assets = AssetPair { asset_in, asset_out };
		let fee = (asset_out, amount_out.fee);
		let amount_in_without_fee =
			amount_in
				.amount
				.checked_sub(amount_in.fee)
				.ok_or(ExecutorError::Error(DispatchError::Other(
					"Error while executing sell trade",
				)))?;
		let amount_out_without_fee = amount_out
			.amount
			.checked_sub(amount_out.fee)
			.ok_or(ExecutorError::Error(DispatchError::Other(
				"Error while executing sell trade",
			)))?;

		let amm_transfer: AMMTransfer<T::AccountId, AssetId, AssetPair, Balance> = AMMTransfer {
			origin: who.clone(),
			assets,
			amount: amount_in_without_fee,
			amount_out: amount_out_without_fee,
			fee,
			discount: false,
			discount_amount: 0,
		};
		Self::do_validate_sell(amount_in_without_fee, assets, &who).map_err(|de| ExecutorError::Error(de))?;
		Self::do_execute_sell(&amm_transfer).map_err(|de| ExecutorError::Error(de))?;

		Ok(())
	}

	fn execute_buy(
		pool_type: PoolType<AssetId>,
		who: &T::AccountId,
		asset_in: AssetId,
		asset_out: AssetId,
		amount_out: Self::TradeCalculationResult,
	) -> Result<(), ExecutorError<Self::Error>> {
		if pool_type != PoolType::XYK {
			return Err(ExecutorError::NotSupported);
		}

		let amount_in = Self::calculate_buy(pool_type, asset_in, asset_out, amount_out)?;

		let assets = AssetPair { asset_in, asset_out };
		let fee = (asset_in, amount_in.fee);
		let amount_out_with_fee = amount_out
			.amount
			.checked_add(amount_out.fee)
			.ok_or(ExecutorError::Error(DispatchError::Other(
				"Error while executing buy trade",
			)))?;

		let amm_transfer: AMMTransfer<T::AccountId, AssetId, AssetPair, Balance> = AMMTransfer {
			origin: who.clone(),
			assets,
			amount: amount_out_with_fee,
			amount_out: amount_in.amount,
			fee,
			discount: false,
			discount_amount: 0,
		};
		Self::do_validate_buy(assets, amount_out_with_fee).map_err(|de| ExecutorError::Error(de))?;
		Self::do_execute_buy(&amm_transfer).map_err(|de| ExecutorError::Error(de))?;

		Ok(())
	}
}
