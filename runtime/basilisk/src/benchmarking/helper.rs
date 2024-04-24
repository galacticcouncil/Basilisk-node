use pallet_treasury::ArgumentsFactory;
use primitives::AccountId;

pub struct BenchmarkHelper;

// Support for pallet treasury benchmarking
impl ArgumentsFactory<(), AccountId> for BenchmarkHelper {
	fn create_asset_kind(_seed: u32) -> () {
		()
	}

	fn create_beneficiary(seed: [u8; 32]) -> AccountId {
		AccountId::from(seed)
	}
}
