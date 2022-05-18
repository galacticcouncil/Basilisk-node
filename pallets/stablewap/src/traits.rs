pub trait ShareAccountIdFor<Assets> {
	type AccountId;

	fn from_assets(assets: &Assets, identifier: Option<&str>) -> Self::AccountId;

	fn name(assets: &Assets, identifier: Option<&str>) -> Vec<u8>;
}
