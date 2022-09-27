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
//- refactor unit tests to capture this accumulated/distributed domain logic
//- refactor trade executor, pass the amount in and amount out to execute functins so we can use them as max/min limit
//- abstract away the type check at the beginning
//- parameterize everything asset balance in integrationn tests
//- add builder if possible to integration tests if possible
//- TODO Dani comments

impl<T: Config> TradeExecution<T::Origin, T::AccountId, AssetId, Balance> for Pallet<T> {
    type Error = DispatchError;

    fn calculate_sell(pool_type: PoolType<AssetId>, asset_in: AssetId, asset_out: AssetId, amount_in: Balance) -> Result<Balance, ExecutorError<Self::Error>> {
        if pool_type != PoolType::LBP {
            return Err(ExecutorError::NotSupported);
        }

        let assets = AssetPair{asset_in, asset_out };
        let pool_id = Self::get_pair_id(assets);
        let pool_data = <PoolData<T>>::try_get(&pool_id).map_err(|_| ExecutorError::Error(Error::<T>::PoolNotFound.into()))?;

        let now = T::BlockNumberProvider::current_block_number();
        let (weight_in, weight_out) = Self::get_sorted_weight(assets.asset_in, now, &pool_data).map_err(|err| ExecutorError::Error(err.into()))?;
        let asset_in_reserve = T::MultiCurrency::free_balance(assets.asset_in, &pool_id);
        let asset_out_reserve = T::MultiCurrency::free_balance(assets.asset_out, &pool_id);

        let fee_asset = pool_data.assets.0;

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

            Ok(amount_out) //amount with fee applied as the user is responsible to send fee to the fee collector
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

            Ok(amount_out_without_fee) //amount without fee as the pool is responsible to send fee to the fee collector
        }
    }

    fn calculate_buy(pool_type: PoolType<AssetId>, asset_in: AssetId, asset_out: AssetId, amount_out: Balance) -> Result<Balance, ExecutorError<Self::Error>> {
        if pool_type != PoolType::LBP {
            return Err(ExecutorError::NotSupported);
        }

        let assets = AssetPair {asset_in, asset_out};
        let pool_id = Self::get_pair_id(assets);
        let pool_data = <PoolData<T>>::try_get(&pool_id).map_err(|_| ExecutorError::Error(Error::<T>::PoolNotFound.into()))?;

        let now = T::BlockNumberProvider::current_block_number();
        let (weight_in, weight_out) = Self::get_sorted_weight(assets.asset_in, now, &pool_data).map_err(|err| ExecutorError::Error(err.into()))?;
        let asset_in_reserve = T::MultiCurrency::free_balance(assets.asset_in, &pool_id);
        let asset_out_reserve = T::MultiCurrency::free_balance(assets.asset_out, &pool_id);

        // LBP fee asset is always accumulated asset
        let fee_asset = pool_data.assets.0;

        // Accumulated asset is bought (out) of the pool for distributed asset (in)
        // Take accumulated asset (out) sans fee from the pool and send to seller
        // Take distributed asset (in) from the seller and add to pool
        // Take fee from the pool and send to fee collector
        // Buyer bears repay fee
        if fee_asset == assets.asset_out {
            let fee = Self::calculate_fees(&pool_data, amount_out).map_err(|err| ExecutorError::Error(err.into()))?;
            let amount_out_plus_fee = amount_out.checked_add(fee).ok_or(ExecutorError::Error(Error::<T>::Overflow.into()))?;

            let calculated_in = hydra_dx_math::lbp::calculate_in_given_out(
                asset_in_reserve,
                asset_out_reserve,
                weight_in,
                weight_out,
                amount_out_plus_fee,
            )
                .map_err(|_| ExecutorError::Error(Error::<T>::Overflow.into()))?;

            //TODO: Dani it is strange that we do not need to add the fee, or doing something with it here or in the next as the feecollector receives it from other place
            Ok(calculated_in)

        } else {
            let calculated_in = hydra_dx_math::lbp::calculate_in_given_out(
                asset_in_reserve,
                asset_out_reserve,
                weight_in,
                weight_out,
                amount_out,
            )
                .map_err(|_| ExecutorError::Error(Error::<T>::Overflow.into()))?;

            let fee = Self::calculate_fees(&pool_data, calculated_in).map_err(|err| ExecutorError::Error(err.into()))?;

            Ok(calculated_in)
        }
    }

    fn execute_sell(who: T::Origin, pool_type: PoolType<AssetId>, asset_in: AssetId, asset_out: AssetId, amount_in: Balance) -> Result<(), ExecutorError<Self::Error>> {
        if pool_type != PoolType::LBP {
            return Err(ExecutorError::NotSupported);
        }
        //TODO: Dani we could pass amount_out here and pass it to sell
        Self::sell(who, asset_in, asset_out, amount_in, Balance::zero()).map_err(ExecutorError::Error)
    }

    fn execute_buy(who: T::Origin, pool_type: PoolType<AssetId>, asset_in: AssetId, asset_out: AssetId, amount_out: Balance) -> Result<(), ExecutorError<Self::Error>> {
        if pool_type != PoolType::LBP {
            return Err(ExecutorError::NotSupported);
        }
        Self::buy(who, asset_out,asset_in, amount_out, Balance::MAX).map_err(ExecutorError::Error)
    }
}