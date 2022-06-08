use sp_std::vec::Vec;

/// Abstraction over account id and account name creation for `Assets`
pub trait ShareAccountIdFor<Assets> {
	type AccountId;

	/// Create account id for given assets and an identifier
	fn from_assets(assets: &Assets, identifier: Option<&str>) -> Self::AccountId;

	/// Create a name to uniquely identify a share account id for given assets and an identifier.
	fn name(assets: &Assets, identifier: Option<&str>) -> Vec<u8>;
}
