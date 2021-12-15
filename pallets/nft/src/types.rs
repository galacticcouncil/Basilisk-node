use frame_support::pallet_prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

pub trait NftPermission<InnerClassType> {
	fn can_create(class_type: &InnerClassType) -> bool;
	fn can_mint(class_type: &InnerClassType) -> bool;
	fn can_transfer(class_type: &InnerClassType) -> bool;
	fn can_burn(class_type: &InnerClassType) -> bool;
	fn can_destroy(class_type: &InnerClassType) -> bool;
	fn has_deposit(class_type: &InnerClassType) -> bool;
}

#[impl_trait_for_tuples::impl_for_tuples(5)]
impl<InnerClassType> NftPermission<InnerClassType> for Tuple {
	fn can_create(class_type: &InnerClassType) -> bool {
		for_tuples!( #(
			let result  = match Tuple::can_create(class_type) {
				true => return true,
				false => false,
			};
		)* );
		false
	}

	fn can_mint(class_type: &InnerClassType) -> bool {
		for_tuples!( #(
			let result  = match Tuple::can_create(class_type) {
				true => return true,
				false => false,
			};
		)* );
		false
	}

	fn can_transfer(class_type: &InnerClassType) -> bool {
		for_tuples!( #(
			let result  = match Tuple::can_create(class_type) {
				true => return true,
				false => false,
			};
		)* );
		false
	}

	fn can_burn(class_type: &InnerClassType) -> bool {
		for_tuples!( #(
			let result  = match Tuple::can_create(class_type) {
				true => return true,
				false => false,
			};
		)* );
		false
	}

	fn can_destroy(class_type: &InnerClassType) -> bool {
		for_tuples!( #(
			let result  = match Tuple::can_destroy(class_type) {
				true => return true,
				false => false,
			};
		)* );
		false
	}

	fn has_deposit(class_type: &InnerClassType) -> bool {
		for_tuples!( #(
			let result  = match Tuple::has_deposit(class_type) {
				true => return true,
				false => false,
			};
		)* );
		false
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClassInfo<ClassType, BoundedString> {
	/// The user account which receives the royalty
	pub class_type: ClassType,
	/// Arbitrary data about a class, e.g. IPFS hash
	pub metadata: BoundedString,
}
