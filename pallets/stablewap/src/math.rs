use crate::types::{AssetAmounts, Balance, PoolInfo};
use primitive_types::U256;
use sp_runtime::traits::Zero;

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
	pool: &PoolInfo<AssetId, Balance>,
	initial_reserves: &AssetAmounts<Balance>,
	updated_reserves: &AssetAmounts<Balance>,
	precision: Balance,
	share_issuance: Balance,
) -> Option<AssetAmountChanges<Balance>> {
	let ann = calculate_ann(pool.amplification)?;

	let initial_d = calculate_d(&[initial_reserves.0, initial_reserves.1], ann, precision)?;
	let updated_d = calculate_d(&[updated_reserves.0, updated_reserves.1], ann, precision)?;

	if updated_d <= initial_d {
		return None;
	}

	let share_amount = if share_issuance > Balance::zero() {
		// TODO: fee accounting
		updated_d
	} else {
		updated_d
	};

	Some(AssetAmountChanges { share_amount })
}

pub(crate) struct TradeChanges {
	pub delta_amount_out: Balance,
}

pub(crate) fn calculate_sell_changes<AssetId>(
	_pool: &PoolInfo<AssetId, Balance>,
	_asset_in: AssetId,
	_asset_out: AssetId,
	_amount: Balance,
) -> Option<TradeChanges> {
	Some(TradeChanges {
		delta_amount_out: Balance::zero(),
	})
}

pub(crate) fn calculate_buy_changes<AssetId>(
	_pool: &PoolInfo<AssetId, Balance>,
	_asset_in: AssetId,
	_asset_out: AssetId,
	_amount: Balance,
) -> Option<TradeChanges> {
	Some(TradeChanges {
		delta_amount_out: Balance::zero(),
	})
}

fn calculate_ann(amplification: Balance) -> Option<Balance> {
	let n_coins = Balance::from(NUMBER_OF_ASSETS_PER_POOL as u128);
	(0..NUMBER_OF_ASSETS_PER_POOL).try_fold(amplification, |acc, _| acc.checked_mul(n_coins))
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

fn calculate_y(reserve: Balance, d: Balance, ann: Balance, precision: Balance) -> Option<Balance> {
	let (d_hp, two_hp, n_coins_hp, ann_hp, new_reserve_hp, precision_hp) =
		to_u256!(d, 2u128, NUMBER_OF_ASSETS_PER_POOL, ann, reserve, precision);

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
			.checked_div(two_hp.checked_mul(y)?.checked_add(b)?.checked_sub(d_hp)?)?;

		if y > y_prev {
			if y.checked_sub(y_prev)? <= precision_hp {
				return to_balance!(y);
			}
		} else {
			if y_prev.checked_sub(y)? <= precision_hp {
				return to_balance!(y);
			}
		}
	}

	None
}

#[test]
fn test_ann() {
	assert_eq!(calculate_ann(Balance::from(1u128)), Some(Balance::from(4u128)));
	assert_eq!(calculate_ann(Balance::from(10u128)), Some(Balance::from(40u128)));
	assert_eq!(calculate_ann(Balance::from(100u128)), Some(Balance::from(400u128)));
}

#[test]
fn test_d() {
	let precision = Balance::from(1_u128);

	let reserves = [1000u128, 1000u128];
	let ann = 4u128;
	assert_eq!(calculate_d(&reserves, ann, precision), Some(2000u128));

	let reserves = [1_000_000_000_000_000_000_000u128, 1_000_000_000_000_000_000_000u128];
	let ann = 4u128;
	assert_eq!(
		calculate_d(&reserves, ann, precision),
		Some(2_000_000_000_000_000_000_000u128)
	);
}

#[test]
fn test_y_given_in() {
	let precision = Balance::from(1_u128);
	let reserves = [1000u128, 2000u128];
	let ann = 4u128;

	let amount_in = 100u128;
	assert_eq!(calculate_d(&reserves, ann, precision), Some(2940u128));
	assert_eq!(
		calculate_y_given_in(amount_in, reserves[0], reserves[1], ann, precision),
		Some(2000u128 - 126u128)
	);
	assert_eq!(
		calculate_d(&[1100u128, 2000u128 - 126u128], ann, precision),
		Some(2939u128)
	);
}

#[test]
fn test_y_given_out() {
	let precision = Balance::from(1_u128);
	let reserves = [1000u128, 2000u128];
	let ann = 4u128;

	let amount_out = 100u128;

	let expected_in = 79u128;

	assert_eq!(calculate_d(&reserves, ann, precision), Some(2940u128));

	assert_eq!(
		calculate_y_given_out(amount_out, reserves[0], reserves[1], ann, precision),
		Some(1000u128 + expected_in)
	);
	assert_eq!(
		calculate_d(&[1000u128 + expected_in, 2000u128 - amount_out], ann, precision),
		Some(2939u128)
	);
}
