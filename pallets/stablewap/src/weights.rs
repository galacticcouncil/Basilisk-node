use frame_support::weights::Weight;

pub trait WeightInfo {
	fn create_pool() -> Weight;
}

impl WeightInfo for () {
	fn create_pool() -> Weight {
		0
	}
}
