use frame_support::pallet_prelude::Get;
use frame_support::traits::Contains;
use polkadot_xcm::latest::prelude::*;
use sp_std::marker::PhantomData;
use sp_std::vec::Vec;

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
