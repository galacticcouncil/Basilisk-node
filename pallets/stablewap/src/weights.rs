use frame_support::weights::Weight;

pub trait WeightInfo {
	fn create_pool() -> Weight;
	fn add_liquidity() -> Weight;
	fn remove_liquidity() -> Weight;
	fn sell() -> Weight;
	fn buy() -> Weight;
}

impl WeightInfo for () {
	fn create_pool() -> Weight {
		0
	}

	fn add_liquidity() -> Weight {
		0
	}

	fn remove_liquidity() -> Weight {
		0
	}

	fn sell() -> Weight {
		0
	}

	fn buy() -> Weight {
		0
	}
}
