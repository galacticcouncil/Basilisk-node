use super::*;
use crate::mock::*;
use frame_support::{assert_ok};
use primitives::nft::ClassType;
use sp_std::convert::TryInto;

pub type AuctionsModule = Pallet<Test>;

#[cfg(test)]
mod english;

#[cfg(test)]
mod topup;

#[cfg(test)]
mod candle;

macro_rules! bvec {
	($( $x:tt )*) => {
		vec![$( $x )*].try_into().unwrap()
	}
}

fn new_test_ext() -> sp_io::TestExternalities {
  let mut ext = ExtBuilder::default().build();
  ext.execute_with(|| set_block_number::<Test>(1));
  ext
}

fn predefined_test_ext() -> sp_io::TestExternalities {
  let mut ext = new_test_ext();

  ext.execute_with(|| {
    assert_ok!(Nft::create_class(
      Origin::signed(ALICE),
      NFT_CLASS_ID_1,
      ClassType::Marketplace,
      bvec![0]
    ));
    assert_ok!(Nft::mint(Origin::signed(ALICE), nft_class_id::<Test>(NFT_CLASS_ID_1), nft_instance_id::<Test>(NFT_INSTANCE_ID_1), bvec![0]));
  });

  ext
}
