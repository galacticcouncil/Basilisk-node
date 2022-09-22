use crate::{Config, Error, Pallet, PoolData};
use frame_support::ensure;
use frame_support::traits::Get;
use hydradx_traits::router::{ExecutorError, PoolType, TradeExecution};
use hydradx_traits::AMM;
use orml_traits::MultiCurrency;
use primitives::asset::AssetPair;
use primitives::{AssetId, Balance};
use sp_runtime::traits::{BlockNumberProvider, Zero};
use sp_runtime::DispatchError;

//TODO: Dani
//- add integration test for comparing the router trade with the direct LBP pallet trade
//- refactor integration tests so use BSX and distributed asset
//- refactor unit tests to capture this accumulated/distributed domain logic
//- refactor trade executor, pass the amount in and amount out to execute functins so we can use them as max/min limit

impl<T: Config> TradeExecution<T::Origin, T::AccountId, AssetId, Balance> for Pallet<T> {
    type Error = DispatchError;

    fn calculate_sell(pool_type: PoolType<AssetId>, asset_in: AssetId, asset_out: AssetId, amount_in: Balance) -> Result<Balance, ExecutorError<Self::Error>> {
        if pool_type != PoolType::LBP {
            return Err(ExecutorError::NotSupported);
        }

        /*ensure!(!amount.is_zero(), Error::<T>::ZeroAmount);
        ensure!(
			T::MultiCurrency::free_balance(assets.asset_in, who) >= amount,
			Error::<T>::InsufficientAssetBalance
		);*/

        let assets = AssetPair{asset_in, asset_out };
        let pool_id = Self::get_pair_id(assets);
        let pool_data = <PoolData<T>>::try_get(&pool_id).map_err(|_| ExecutorError::Error(Error::<T>::PoolNotFound.into()))?;

        //ensure!(Self::is_pool_running(&pool_data), Error::<T>::SaleIsNotRunning);

        let now = T::BlockNumberProvider::current_block_number();
        let (weight_in, weight_out) = Self::get_sorted_weight(assets.asset_in, now, &pool_data).map_err(|err| ExecutorError::Error(err.into()))?;
        let asset_in_reserve = T::MultiCurrency::free_balance(assets.asset_in, &pool_id);
        let asset_out_reserve = T::MultiCurrency::free_balance(assets.asset_out, &pool_id);

       /* ensure!(
			amount <= asset_in_reserve.checked_div(MAX_IN_RATIO).ok_or(Error::<T>::Overflow)?,
			Error::<T>::MaxInRatioExceeded
		);*/

        // LBP fee asset is always accumulated asset
        let fee_asset = pool_data.assets.0;

        // Accumulated asset is sold (in) to the pool for distributed asset (out)
        // Take accumulated asset (in) sans fee from the seller and add to pool
        // Take distributed asset (out) and send to seller
        // Take fee from the seller and send to fee collector
        // Pool bears repay fee
        if fee_asset == assets.asset_in {
            let fee = Self::calculate_fees(&pool_data, amount_in).map_err(|err| ExecutorError::Error(err.into()))?;

            let amount_out = hydra_dx_math::lbp::calculate_out_given_in(
                asset_in_reserve,
                asset_out_reserve,
                weight_in,
                weight_out,
                amount_in,
            )
                .map_err(|_| ExecutorError::Error(Error::<T>::Overflow.into()))?;

           /* ensure!(
				amount_out
					<= asset_out_reserve
						.checked_div(MAX_OUT_RATIO)
						.ok_or(Error::<T>::Overflow)?,
				Error::<T>::MaxOutRatioExceeded
			);

            ensure!(min_bought <= amount_out, Error::<T>::TradingLimitReached);*/

            let amount_without_fee = amount_in.checked_sub(fee).ok_or(ExecutorError::Error(Error::<T>::Overflow.into()))?;

            //TODO: a fee-t kikene vonni?
            //let amount_out_without_fee = amount_out.checked_sub(fee).ok_or(ExecutorError::Error(Error::<T>::Overflow.into()))?;

            //TODO: add comment why we need keep the fee inside
            Ok(amount_out) //

            /*Ok(AMMTransfer {
                origin: who.clone(),
                assets,
                amount: amount_without_fee,
                amount_out,
                discount: false,
                discount_amount: 0_u128,
                fee: (fee_asset, fee),
            })*/

            // Distributed asset is sold (in) to the pool for accumulated asset (out)
            // Take accumulated asset (out) from the pool sans fee and send to the seller
            // Take distributed asset (in) from the seller and send to pool
            // Take fee from the pool and send to fee collector
            // Seller bears repay fee
        } else {
            let calculated_out = hydra_dx_math::lbp::calculate_out_given_in(
                asset_in_reserve,
                asset_out_reserve,
                weight_in,
                weight_out,
                amount_in,
            )
                .map_err(|_| ExecutorError::Error(Error::<T>::Overflow.into()))?;

            let fee = Self::calculate_fees(&pool_data, calculated_out).map_err(|err| ExecutorError::Error(err.into()))?;
            let amount_out_without_fee = calculated_out.checked_sub(fee).ok_or(ExecutorError::Error(Error::<T>::Overflow.into()))?;

            /*ensure!(
				calculated_out
					<= asset_out_reserve
						.checked_div(MAX_OUT_RATIO)
						.ok_or(Error::<T>::Overflow)?,
				Error::<T>::MaxOutRatioExceeded
			);

            ensure!(min_bought <= amount_out_without_fee, Error::<T>::TradingLimitReached);*/

            Ok(amount_out_without_fee)

            /*Ok(AMMTransfer {
                origin: who.clone(),
                assets,
                amount,
                amount_out: amount_out_without_fee,
                discount: false,
                discount_amount: 0_u128,
                fee: (fee_asset, fee),
            })*/
        }
    }

    fn calculate_buy(pool_type: PoolType<AssetId>, asset_in: AssetId, asset_out: AssetId, amount_out: Balance) -> Result<Balance, ExecutorError<Self::Error>> {
        if pool_type != PoolType::LBP {
            return Err(ExecutorError::NotSupported);
        }
        todo!()
    }

    fn execute_sell(who: T::Origin, pool_type: PoolType<AssetId>, asset_in: AssetId, asset_out: AssetId, amount_in: Balance) -> Result<(), ExecutorError<Self::Error>> {
        if pool_type != PoolType::LBP {
            return Err(ExecutorError::NotSupported);
        }
        //we could pass amount_out here and pass it to sell
        Self::sell(who, asset_in, asset_out, amount_in, Balance::zero()).map_err(ExecutorError::Error)
    }

    fn execute_buy(who: T::Origin, pool_type: PoolType<AssetId>, asset_in: AssetId, asset_out: AssetId, amount_out: Balance) -> Result<(), ExecutorError<Self::Error>> {
        if pool_type != PoolType::LBP {
            return Err(ExecutorError::NotSupported);
        }
        todo!()
    }
}