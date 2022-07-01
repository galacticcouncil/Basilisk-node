use frame_support::pallet_prelude::Get;
use frame_support::traits::Contains;
use polkadot_xcm::latest::prelude::*;
use sp_std::marker::PhantomData;
use sp_std::vec::Vec;

/// A filter that allows all assets except assets with and id corresponding to the excluded location.
pub struct AllowEverythingExcept<ExcludedLocation>(PhantomData<ExcludedLocation>);
impl<ExcludedLocation: Get<MultiLocation>> Contains<(MultiLocation, Vec<MultiAsset>)>
	for AllowEverythingExcept<ExcludedLocation>
{
	fn contains((_location, assets): &(MultiLocation, Vec<MultiAsset>)) -> bool {
		!assets
			.iter()
			.any(|asset| asset.is_fungible(Some(Concrete(ExcludedLocation::get()))))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn allow_everything_except() {
		frame_support::parameter_types! {
			pub ParentLocation: MultiLocation = MultiLocation::parent();
		}
		type DontAllowParent = AllowEverythingExcept<ParentLocation>;

		assert!(DontAllowParent::contains(&(
			MultiLocation::here(),
			vec![
				(MultiLocation::here(), 1234).into(),
				(MultiLocation::new(1, X1(Parachain(2000))), 42).into()
			]
		)));
		assert!(!DontAllowParent::contains(&(
			MultiLocation::here(),
			vec![(MultiLocation::parent(), 1234).into()]
		)))
	}
}
