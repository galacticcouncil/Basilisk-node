use frame_support::traits::{Currency, Imbalance, OnUnbalanced};
use pallet_balances::NegativeImbalance;

use super::AccountId;

/// Logic for the author to get a portion of fees.
pub struct ToAuthor<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for ToAuthor<R>
where
        R: pallet_balances::Config + pallet_authorship::Config,
        <R as frame_system::Config>::AccountId: From<AccountId>,
        <R as frame_system::Config>::AccountId: Into<AccountId>,
        <R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
        fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
                let numeric_amount = amount.peek();
                let author = <pallet_authorship::Pallet<R>>::author();
                <pallet_balances::Pallet<R>>::resolve_creating(
                        &<pallet_authorship::Pallet<R>>::author(),
                        amount,
                );
                <frame_system::Pallet<R>>::deposit_event(pallet_balances::Event::Deposit(
                        author,
                        numeric_amount,
                ));
        }
}
