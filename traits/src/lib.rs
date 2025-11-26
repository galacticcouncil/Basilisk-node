#![cfg_attr(not(feature = "std"), no_std)]

pub mod oracle;
pub mod router;


use codec::{Decode, Encode};
use serde::{Deserialize, Serialize};
use frame_support::sp_runtime::{traits::Zero, RuntimeDebug};
use frame_support::weights::Weight;

