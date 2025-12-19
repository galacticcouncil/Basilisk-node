// Runtimes with enabled `runtime-benchmarks` feature behave differently.
// Disable the integration tests when this feature is enabled.
#![cfg(not(feature = "runtime-benchmarks"))]
mod call_filter;
mod fees;
mod kusama_test_net;
mod nft;
mod nft_marketplace;
mod non_native_fee;
mod oracle;
mod router;
mod sessions;
mod transact_call_filter;
mod vesting;
mod xyk;
