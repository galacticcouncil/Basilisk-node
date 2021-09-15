pub use basilisk_runtime::{
	AccountId, AssetId, AssetLocation, AssetRegistry, Balance, Balances, Origin, Runtime, System, Tokens, XTokens, BSX,
};
use basilisk_runtime::{NativeAssetId, NativeExistentialDeposit};
use frame_support::traits::GenesisBuild;

pub const ALICE: [u8; 32] = [4u8; 32];
pub const BOB: [u8; 32] = [5u8; 32];

pub struct ExtBuilder {
	balances: Vec<(AccountId, AssetId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { balances: vec![] }
	}
}

impl ExtBuilder {
	pub fn balances(mut self, balances: Vec<(AccountId, AssetId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		let native_asset_id = NativeAssetId::get();
		let existential_deposit = NativeExistentialDeposit::get();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self
				.balances
				.clone()
				.into_iter()
				.filter(|(_, asset_id, _)| *asset_id == native_asset_id)
				.map(|(account_id, _, initial_balance)| (account_id, initial_balance))
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			balances: self
				.balances
				.into_iter()
				.filter(|(_, asset_id, _)| *asset_id != native_asset_id)
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		pallet_asset_registry::GenesisConfig::<Runtime> {
			asset_names: vec![(b"KSM".to_vec(), 1u128)],
			native_asset_name: b"BSX".to_vec(),
			native_existential_deposit: existential_deposit,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
