use frame_support::pallet_prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BondingCurve {
    /// The exponent of the curve.
    exponent: u32,
    /// The slope of the curve.
    slope: u128,
    /// The maximum supply that can be minted from the curve.
    max_supply: u128,
}

impl BondingCurve {
    /// Integral when the curve is at point `x`.
    pub fn integral(&self, x: u128) -> u128 {
        let nexp = self.exponent + 1;
        x.pow(nexp) * self.slope / nexp as u128
    }
}