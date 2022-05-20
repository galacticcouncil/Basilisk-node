use crate::types::{Balance, FixedBalance, PoolInfo};
use primitive_types::U256;
use sp_runtime::traits::{CheckedMul, Zero};
use sp_runtime::FixedU128;

const NUMBER_OF_ASSETS_PER_POOL: u128 = 2;

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

pub(crate) struct AssetAmountChanges<Balance> {
	pub share_amount: Balance,
}

pub(crate) fn calculate_add_liquidity_changes<AssetId>(
	_pool: &PoolInfo<AssetId, FixedBalance>,
	_asset: AssetId,
	_reserve: Balance,
	_amount: Balance,
) -> Option<AssetAmountChanges<Balance>> {
	None
}

pub(crate) struct TradeChanges {
	pub delta_amount_out: Balance,
}

pub(crate) fn calculate_sell_changes<AssetId>(
	_pool: &PoolInfo<AssetId, FixedBalance>,
	_asset_in: AssetId,
	_asset_out: AssetId,
	_amount: Balance,
) -> Option<TradeChanges> {
	Some(TradeChanges {
		delta_amount_out: Balance::zero(),
	})
}

pub(crate) fn calculate_buy_changes<AssetId>(
	_pool: &PoolInfo<AssetId, FixedBalance>,
	_asset_in: AssetId,
	_asset_out: AssetId,
	_amount: Balance,
) -> Option<TradeChanges> {
	Some(TradeChanges {
		delta_amount_out: Balance::zero(),
	})
}

fn calculate_ann(amplification: FixedBalance) -> Option<FixedBalance> {
	let n_coins = FixedBalance::from(NUMBER_OF_ASSETS_PER_POOL as u128);
	(0..NUMBER_OF_ASSETS_PER_POOL).try_fold(amplification, |acc, _| acc.checked_mul(&n_coins))
}

fn calculate_d(xp: &[Balance; 2], ann: Balance, precision: Balance) -> Option<Balance> {
	let n_coins = NUMBER_OF_ASSETS_PER_POOL;

	let xp_hp: [U256; 2] = [to_u256!(xp[0]), to_u256!(xp[1])];

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
			)?;

		if d > d_prev {
			if d.checked_sub(d_prev)? <= precision_hp {
				return to_balance!(d);
			}
		} else {
			if d_prev.checked_sub(d)? <= precision_hp {
				return to_balance!(d);
			}
		}
	}
	None
}

fn calculate_y(reserve_in: Balance, reserve_out: Balance, ann: Balance) -> Option<Balance> {
	let prec = Balance::from(10_000_u128);
	let zero = Balance::zero();
	let two = Balance::from(2_u128);
	let n_coins = Balance::from(NUMBER_OF_ASSETS_PER_POOL);

	let d = calculate_d(&[reserve_in, reserve_out], ann, prec)?;

	let mut c = d;
	let mut s = zero;
	s = s.checked_add(reserve_in)?;
	c = c.checked_mul(d)?.checked_div(reserve_in.checked_mul(2u128)?)?;
	c = c.checked_mul(d)?.checked_div(ann.checked_mul(n_coins)?)?;
	let b = s.checked_add(d.checked_div(ann)?)?;
	let mut y = d;

	for _ in 0..255 {
		let y_prev = y;
		y = y
			.checked_mul(y)?
			.checked_add(c)?
			.checked_div(two.checked_mul(y)?.checked_add(b)?.checked_sub(d)?)?;

		if y > y_prev {
			if y.checked_sub(y_prev)? <= prec {
				return Some(y);
			}
		} else {
			if y_prev.checked_sub(y)? <= prec {
				return Some(y);
			}
		}
	}

	None
}

#[cfg(test)]
#[test]
fn test_ann() {
	assert_eq!(calculate_ann(FixedU128::from(1u128)), Some(FixedBalance::from(4u128)));
	assert_eq!(
		calculate_ann(FixedU128::from_float(0.5)),
		Some(FixedBalance::from(2u128))
	);
	assert_eq!(
		calculate_ann(FixedU128::from_float(100.0)),
		Some(FixedBalance::from(400u128))
	);
}

#[cfg(test)]
#[test]
fn test_d() {
	let precision = Balance::from(10_000_u128);

	let reserves = [1000u128, 1000u128];
	let ann = 4u128;
	assert_eq!(calculate_d(&reserves, ann, precision), Some(2000u128));

	let reserves = [1000_000_000_000_000_000_000u128, 1000_000_000_000_000_000_000u128];
	let ann = 4u128;
	assert_eq!(
		calculate_d(&reserves, ann, precision),
		Some(2000_000_000_000_000_000_000u128)
	);
}

#[cfg(test)]
#[test]
fn test_y() {
	let reserves = [1000u128, 2000u128];
	let ann = 4u128;
	assert_eq!(calculate_y(reserves[0], reserves[1], ann), Some(2189u128));
	assert_eq!(calculate_y(reserves[1], reserves[0], ann), Some(1663u128));
}
