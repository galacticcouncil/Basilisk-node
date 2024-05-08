// Runtimes with enabled `runtime-benchmarks` feature behave differently.
// Disable the integration tests when this feature is enabled.
#![cfg(not(feature = "runtime-benchmarks"))]
mod call_filter;
mod cross_chain_transfer;
mod exchange_asset;
mod fees;
mod kusama_test_net;
mod nft;
mod nft_marketplace;
mod non_native_fee;
mod oracle;
mod router;
mod transact_call_filter;
mod vesting;
mod xyk;
