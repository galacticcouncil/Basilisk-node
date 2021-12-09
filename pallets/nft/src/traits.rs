
pub trait NftPermission<InnerClassType> {
    fn can_create(class_type: &InnerClassType) -> bool;
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
}