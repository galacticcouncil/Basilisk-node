//! # Basilisk Math
//!
//! A collection of utilities to make performing liquidity pool
//! calculations more convenient.
#![allow(clippy::bool_to_int_with_if)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), test))]
extern crate std;

pub mod ema;
