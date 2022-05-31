use primitive_types::U256;

#[macro_export]
macro_rules! to_u256 {
    ($($x:expr),+) => (
        {($(U256::from($x)),+)}
    );
}

#[macro_export]
macro_rules! to_balance {
	($x:expr) => {
		Balance::try_from($x).ok()
	};
}

/// Stableswap/curve math reduced to two assets.
pub(crate) mod two_asset_pool_math {

	use super::*;
	use crate::types::{AssetAmounts, Balance};
	use sp_runtime::traits::Zero;

	/// Calculate shares amount after liquidity is added to the pool.
	///
	/// No fee applied. Currently is expected that liquidity of both assets are added to the pool.
	///
	/// share_amount = share_supply * ( d1 - d0 ) / d0
	///
	/// Returns `Some(shares)` when successful.
	pub(crate) fn calculate_add_liquidity_shares(
		initial_reserves: &AssetAmounts<Balance>,
		updated_reserves: &AssetAmounts<Balance>,
		precision: Balance,
		amplification: Balance,
		share_issuance: Balance,
	) -> Option<Balance> {
		let ann = calculate_ann(amplification)?;

		let initial_d = calculate_d(&[initial_reserves.0, initial_reserves.1], ann, precision)?;
		// We must make sure the updated_d is rounded *down* so that we are not giving the new position too many shares.
		// calculate_d can return a D value that is above the correct D value by up to 2, so we subtract 2.
		let updated_d = calculate_d(&[updated_reserves.0, updated_reserves.1], ann, precision)?.checked_sub(2_u128)?;

		if updated_d < initial_d {
			return None;
		}

		if share_issuance == 0 {
			// if first liquidity added
			Some(updated_d)
		} else {
			let (issuance_hp, d_diff, d0) = to_u256!(share_issuance, updated_d - initial_d, initial_d);
			let share_amount = issuance_hp.checked_mul(d_diff)?.checked_div(d0)?;
			to_balance!(share_amount)
		}
	}

	/// Calculate new reserve of asset b so that the ratio does not change:
	///
	/// new_reserve_b = (reserve_a + amount) * reserve_b / reserve_a
	///
	pub(crate) fn calculate_asset_b_reserve(
		asset_a_reserve: Balance,
		asset_b_reserve: Balance,
		updated_reserve: Balance,
	) -> Option<Balance> {
		let (reserve_a, reserve_b, updated_reserve_b) = to_u256!(asset_a_reserve, asset_b_reserve, updated_reserve);
		let result = reserve_a.checked_mul(updated_reserve_b)?.checked_div(reserve_b)?;
		to_balance!(result)
	}

	/// Given amount of shares and asset reserves, calculate corresponding amounts of each asset to be withdrawn.
	pub(crate) fn calculate_remove_liquidity_amounts(
		reserves: &AssetAmounts<Balance>,
		shares: Balance,
		share_asset_issuance: Balance,
	) -> Option<AssetAmounts<Balance>> {
		let (shares_hp, issuance_hp) = to_u256!(shares, share_asset_issuance);

		let calculate_amount = |asset_reserve: Balance| {
			to_balance!(to_u256!(asset_reserve)
				.checked_mul(shares_hp)?
				.checked_div(issuance_hp)?)
		};

		let amount_a = calculate_amount(reserves.0)?;
		let amount_b = calculate_amount(reserves.1)?;

		Some((amount_a, amount_b).into())
	}

	pub(crate) fn calculate_out_given_in(
		reserve_in: Balance,
		reserve_out: Balance,
		amount_in: Balance,
		precision: Balance,
		amplification: Balance,
	) -> Option<Balance> {
		let ann = calculate_ann(amplification)?;
		let new_reserve_out = calculate_y_given_in(amount_in, reserve_in, reserve_out, ann, precision)?;
		reserve_out.checked_sub(new_reserve_out)
	}

	pub(crate) fn calculate_in_given_out(
		reserve_in: Balance,
		reserve_out: Balance,
		amount_out: Balance,
		precision: Balance,
		amplification: Balance,
	) -> Option<Balance> {
		let ann = calculate_ann(amplification)?;
		let new_reserve_in = calculate_y_given_out(amount_out, reserve_in, reserve_out, ann, precision)?;
		new_reserve_in.checked_sub(reserve_in)
	}

	fn calculate_ann(amplification: Balance) -> Option<Balance> {
		(0..2).try_fold(amplification, |acc, _| acc.checked_mul(2u128))
	}

	/// Calculate `d` so the Stableswap invariant does not change.
	///
	/// Note: this works for two asset pools only!
	///
	/// This is solved using newtons formula by iterating the following equation until convergence.
	///
	/// dn+1 = (ann * S + n * Dp) * dn) / ( (ann -1) * dn + (n+1) * dp)
	///
	/// where
	///
	/// S = sum(xp)
	/// dp = d^n+1 / prod(sp)
	///
	/// if (dn+1 - dn) <= precision - converged successfully.
	///
	/// Parameters:
	/// - `xp`: reserves of asset a and b.
	/// - `ann`: amplification coefficient multiplied by `2^2` ( number of assets in pool)
	/// - `precision`:  convergence precision
	pub(crate) fn calculate_d(xp: &[Balance; 2], ann: Balance, precision: Balance) -> Option<Balance> {
		let two_u256 = to_u256!(2_u128);
		let n_coins = two_u256;

		let mut xp_hp: [U256; 2] = [to_u256!(xp[0]), to_u256!(xp[1])];
		xp_hp.sort();

		let s_hp = xp_hp.iter().try_fold(U256::zero(), |acc, v| acc.checked_add(*v))?;

		if s_hp == U256::zero() {
			return Some(Balance::zero());
		}

		let mut d = s_hp;

		let (n_coins_hp, ann_hp, precision_hp) = to_u256!(n_coins, ann, precision);

		for _ in 0..255 {
			let d_p = xp_hp
				.iter()
				.try_fold(d, |acc, v| acc.checked_mul(d)?.checked_div(v.checked_mul(n_coins_hp)?))?;
			let d_prev = d;

			d = ann_hp
				.checked_mul(s_hp)?
				.checked_add(d_p.checked_mul(n_coins_hp)?)?
				.checked_mul(d)?
				.checked_div(
					ann_hp
						.checked_sub(U256::one())?
						.checked_mul(d)?
						.checked_add(n_coins_hp.checked_add(U256::one())?.checked_mul(d_p)?)?,
				)?
				.checked_add(two_u256)?; // adding two here is sufficient to account for rounding
						 // errors, AS LONG AS the minimum reserves are 2 for each
						 // asset. I.e., as long as xp_hp[0] >= 2 and xp_hp[1] >= 2

			// adding two guarantees that this function will return
			// a value larger than or equal to the correct D invariant

			if d > d_prev {
				if d.checked_sub(d_prev)? <= precision_hp {
					return to_balance!(d);
				}
			} else if d_prev.checked_sub(d)? <= precision_hp {
				return to_balance!(d);
			}
		}
		None
	}

	/// Calculate new amount of reserve OUT given amount to be added to the pool
	fn calculate_y_given_in(
		amount: Balance,
		reserve_in: Balance,
		reserve_out: Balance,
		ann: Balance,
		precision: Balance,
	) -> Option<Balance> {
		let new_reserve_in = reserve_in.checked_add(amount)?;

		let d = calculate_d(&[reserve_in, reserve_out], ann, precision)?;

		calculate_y(new_reserve_in, d, ann, precision)
	}

	/// Calculate new amount of reserve IN given amount to be withdrawn from the pool
	fn calculate_y_given_out(
		amount: Balance,
		reserve_in: Balance,
		reserve_out: Balance,
		ann: Balance,
		precision: Balance,
	) -> Option<Balance> {
		let new_reserve_out = reserve_out.checked_sub(amount)?;

		let d = calculate_d(&[reserve_in, reserve_out], ann, precision)?;

		calculate_y(new_reserve_out, d, ann, precision)
	}

	/// Calculate new reserve amount of an asset given updated reserve of secondary asset and initial `d`
	///
	/// This is solved using Newton's method by iterating the following until convergence:
	///
	/// yn+1 = (yn^2 + c) / ( 2 * yn + b - d )
	///
	/// where
	/// c = d^n+1 / n^n * P * ann
	/// b = s + d/ann
	///
	/// note: thw s and P are sum or prod of all reserves except the one we are calculating but since we are in 2 asset pool - it is just one
	/// s = reserve
	/// P = reserve
	///
	/// Note: this implementation works only for 2 assets pool!
	fn calculate_y(reserve: Balance, d: Balance, ann: Balance, precision: Balance) -> Option<Balance> {
		let (d_hp, two_hp, ann_hp, new_reserve_hp, precision_hp) = to_u256!(d, 2u128, ann, reserve, precision);

		let n_coins_hp = two_hp;
		let s = new_reserve_hp;
		let mut c = d_hp;

		c = c.checked_mul(d_hp)?.checked_div(new_reserve_hp.checked_mul(two_hp)?)?;

		c = c.checked_mul(d_hp)?.checked_div(ann_hp.checked_mul(n_coins_hp)?)?;

		let b = s.checked_add(d_hp.checked_div(ann_hp)?)?;
		let mut y = d_hp;

		for _ in 0..255 {
			let y_prev = y;
			y = y
				.checked_mul(y)?
				.checked_add(c)?
				.checked_div(two_hp.checked_mul(y)?.checked_add(b)?.checked_sub(d_hp)?)?
				.checked_add(two_hp)?;
			// Adding 2 guarantees that at each iteration, we are rounding so as to *overestimate* compared
			// to exact division.
			// Note that while this should guarantee convergence when y is decreasing, it may cause
			// issues when y is increasing.

			if y > y_prev {
				if y.checked_sub(y_prev)? <= precision_hp {
					return to_balance!(y);
				}
			} else if y_prev.checked_sub(y)? <= precision_hp {
				return to_balance!(y);
			}
		}

		None
	}

	#[test]
	fn test_ann() {
		assert_eq!(calculate_ann(1u128), Some(4u128));
		assert_eq!(calculate_ann(10u128), Some(40u128));
		assert_eq!(calculate_ann(100u128), Some(400u128));
	}

	#[test]
	fn test_d() {
		let precision = 1_u128;

		let reserves = [1000u128, 1000u128];
		let ann = 4u128;
		assert_eq!(calculate_d(&reserves, ann, precision), Some(2000u128 + 2u128));

		let reserves = [1_000_000_000_000_000_000_000u128, 1_000_000_000_000_000_000_000u128];
		let ann = 4u128;
		assert_eq!(
			calculate_d(&reserves, ann, precision),
			Some(2_000_000_000_000_000_000_000u128 + 2u128)
		);
	}

	#[test]
	fn test_y_given_in() {
		let precision = 1_u128;
		let reserves = [1000u128, 2000u128];
		let ann = 4u128;

		let amount_in = 100u128;
		assert_eq!(calculate_d(&reserves, ann, precision), Some(2942u128));
		assert_eq!(
			calculate_y_given_in(amount_in, reserves[0], reserves[1], ann, precision),
			Some(2000u128 - 121u128)
		);
		assert_eq!(
			calculate_d(&[1100u128, 2000u128 - 125u128], ann, precision),
			Some(2942u128)
		);
	}

	#[test]
	fn test_y_given_out() {
		let precision = 1_u128;
		let reserves = [1000u128, 2000u128];
		let ann = 4u128;

		let amount_out = 100u128;

		let expected_in = 83u128;

		assert_eq!(calculate_d(&reserves, ann, precision), Some(2942u128));

		assert_eq!(
			calculate_y_given_out(amount_out, reserves[0], reserves[1], ann, precision),
			Some(1000u128 + expected_in)
		);
		assert_eq!(
			calculate_d(&[1000u128 + expected_in, 2000u128 - amount_out], ann, precision),
			Some(2946u128)
		);
	}

	#[test]
	fn test_d_case() {
		let amp = 400u128;
		let ann = amp * 4u128;

		let precision = 1u128;

		let result = calculate_d(&[500000000000008580273458u128, 10u128], ann, precision);

		assert!(result.is_some());
	}

	#[test]
	fn test_d_case2() {
		let amp = 168u128;
		let ann = amp * 4u128;

		let precision = 1u128;

		let result = calculate_d(&[500000000000000000000010u128, 11u128], ann, precision);

		assert!(result.is_some());
	}
}
