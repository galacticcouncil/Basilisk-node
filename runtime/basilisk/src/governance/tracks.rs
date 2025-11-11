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

//! Track configurations for governance.

use super::*;
use primitives::constants::{
	currency::UNITS,
	time::{HOURS, MINUTES},
};
use sp_runtime::{Cow, str_array as s};
const fn percent(x: i32) -> sp_arithmetic::FixedI64 {
	sp_arithmetic::FixedI64::from_rational(x as u128, 100)
}

use pallet_referenda::{Curve, Track, TrackInfo};
const APP_LINEAR: Curve = Curve::make_linear(7, 7, percent(50), percent(100));
const APP_LINEAR_FLAT: Curve = Curve::make_linear(4, 7, percent(50), percent(100));
const APP_RECIP: Curve = Curve::make_reciprocal(1, 7, percent(80), percent(50), percent(100));
const SUP_LINEAR: Curve = Curve::make_linear(7, 7, percent(0), percent(50));
const SUP_RECIP: Curve = Curve::make_reciprocal(5, 7, percent(1), percent(0), percent(50));
const SUP_FAST_RECIP: Curve = Curve::make_reciprocal(3, 7, percent(1), percent(0), percent(50));
const SUP_WHITELISTED_CALLER: Curve = Curve::make_linear(1, 7, percent(0), percent(1));

const TRACKS_DATA: [Track<u16, Balance, BlockNumber>; 8] = [
	Track {
		id: 0,
		info: TrackInfo {
			name: s("root"),
			max_deciding: 3,
			decision_deposit: 100_000_000 * UNITS,
			prepare_period: HOURS,
			decision_period: 7 * DAYS,
			confirm_period: 12 * HOURS,
			min_enactment_period: 10 * MINUTES,
			min_approval: APP_RECIP,
			min_support: SUP_LINEAR,
		},
	},
	Track {
		id: 1,
		info: TrackInfo {
			name: s("whitelisted_caller"),
			max_deciding: 3,
			decision_deposit: 1_000_000 * UNITS,
			prepare_period: 10 * MINUTES,
			decision_period: DAYS,
			confirm_period: 4 * HOURS,
			min_enactment_period: 10 * MINUTES,
			min_approval: APP_RECIP,
			min_support: SUP_WHITELISTED_CALLER,
		},
	},
	Track {
		id: 2,
		info: TrackInfo {
			name: s("referendum_canceller"),
			max_deciding: 10,
			decision_deposit: 10_000_000 * UNITS,
			prepare_period: 60 * MINUTES,
			decision_period: 3 * DAYS,
			confirm_period: 60 * MINUTES,
			min_enactment_period: 10 * MINUTES,
			min_approval: APP_LINEAR_FLAT,
			min_support: SUP_FAST_RECIP,
		},
	},
	Track {
		id: 3,
		info: TrackInfo {
			name: s("referendum_killer"),
			max_deciding: 10,
			decision_deposit: 50_000_000 * UNITS,
			prepare_period: 60 * MINUTES,
			decision_period: 3 * DAYS,
			confirm_period: 3 * HOURS,
			min_enactment_period: 10 * MINUTES,
			min_approval: APP_LINEAR_FLAT,
			min_support: SUP_FAST_RECIP,
		},
	},
	Track {
		id: 4,
		info: TrackInfo {
			name: s("general_admin"),
			max_deciding: 10,
			decision_deposit: 10_000_000 * UNITS,
			prepare_period: 60 * MINUTES,
			decision_period: 7 * DAYS,
			confirm_period: 3 * HOURS,
			min_enactment_period: 10 * MINUTES,
			min_approval: APP_RECIP,
			min_support: SUP_RECIP,
		},
	},
	Track {
		id: 5,
		info: TrackInfo {
			name: s("treasurer"),
			max_deciding: 10,
			decision_deposit: 50_000_000 * UNITS,
			prepare_period: 60 * MINUTES,
			decision_period: 7 * DAYS,
			confirm_period: 12 * HOURS,
			min_enactment_period: 10 * MINUTES,
			min_approval: APP_RECIP,
			min_support: SUP_LINEAR,
		},
	},
	Track {
		id: 6,
		info: TrackInfo {
			name: s("spender"),
			max_deciding: 10,
			decision_deposit: 5_000_000 * UNITS,
			prepare_period: 60 * MINUTES,
			decision_period: 7 * DAYS,
			confirm_period: 3 * HOURS,
			min_enactment_period: 10 * MINUTES,
			min_approval: APP_LINEAR,
			min_support: SUP_RECIP,
		},
	},
	Track {
		id: 7,
		info: TrackInfo {
			name: s("tipper"),
			max_deciding: 10,
			decision_deposit: 500_000 * UNITS,
			prepare_period: 60 * MINUTES,
			decision_period: 7 * DAYS,
			confirm_period: 3 * HOURS,
			min_enactment_period: 10 * MINUTES,
			min_approval: APP_LINEAR_FLAT,
			min_support: SUP_FAST_RECIP,
		},
	},
];

pub struct TracksInfo;
impl pallet_referenda::TracksInfo<Balance, BlockNumber> for TracksInfo {
	type Id = u16;
	type RuntimeOrigin = <RuntimeOrigin as frame_support::traits::OriginTrait>::PalletsOrigin;
	fn tracks() -> impl Iterator<Item = Cow<'static, Track<Self::Id, Balance, BlockNumber>>> {
		TRACKS_DATA.iter().map(Cow::Borrowed)
	}

	fn track_for(id: &Self::RuntimeOrigin) -> Result<Self::Id, ()> {
		if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			match system_origin {
				frame_system::RawOrigin::Root => Ok(0),
				_ => Err(()),
			}
		} else if let Ok(custom_origin) = origins::Origin::try_from(id.clone()) {
			match custom_origin {
				origins::Origin::WhitelistedCaller => Ok(1),
				origins::Origin::ReferendumCanceller => Ok(2),
				origins::Origin::ReferendumKiller => Ok(3),
				origins::Origin::GeneralAdmin => Ok(4),
				origins::Origin::Treasurer => Ok(5),
				origins::Origin::Spender => Ok(6),
				origins::Origin::Tipper => Ok(7),
			}
		} else {
			Err(())
		}
	}
}
