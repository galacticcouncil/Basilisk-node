#![cfg(test)]

use super::*;
use mock::*;
use sp_arithmetics::Percent;

#[test]
fn slash_loyalty_weight_should_work() {
	ExtBuilder::default().build().execute_with(|| {
        // slash 50%
        assert_eq!(Rewards::slash_loyalty_weight(10, 20, Percent::from_percent(50)), 15);

        //slash 0%
        assert_eq!(Rewards::slash_loyalty_weight(125_864_754, 225_864_754, Percent::from_percent(0)), 125_864_754);

        //slash 1%
        assert_eq!(Rewards::slash_loyalty_weight(125_864_754, 213_551_741, Percent::from_percent(1)), 126_741_623);

        //slash 100%
        assert_eq!(Rewards::slash_loyalty_weight(125_864_754, 213_551_741, Percent::from_percent(100)), 213_551_741);
        
        // slash  37%
        assert_eq!(Rewards::slash_loyalty_weight(458_796, 458_983, Percent::from_percent(37)), 458_865);
       
        // slash 255% => same as 100% slash 
        assert_eq!(Rewards::slash_loyalty_weight(100, 200, Percent::from_percent(255)), 200);
    });
}


#[test]
fn get_loyalty_weight_for_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Rewards::get_loyalty_weight_for(1, 1, 1).unwrap(), 0);
        
        assert_eq!(Rewards::get_loyalty_weight_for(1, 1, 1_000).unwrap(), 0);

        assert_eq!(Rewards::get_loyalty_weight_for(1, 10, 1).unwrap(), 9);
        
        assert_eq!(Rewards::get_loyalty_weight_for(1_234, 8_244, 2).unwrap(), 49_140_100);
        
    });
}

/*
#[test]
fn get_loyalty_weight_for_should_not_work() {

}

#[test]
fn get_weighted_shares_should_work() {

}

#[test]
fn get_weighted_shares_should_not_work() {

}

#[test]
fn get_weighted_rewards_should_work() {

}

#[test]
fn get_weighted_rewards_should_not_work() {

}
*/
