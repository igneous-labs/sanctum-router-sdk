use sanctum_marinade_liquid_staking_core::MarinadeError;
use sanctum_reserve_core::ReserveError;
use sanctum_spl_stake_pool_core::SplStakePoolError;
use solido_legacy_core::LidoError;

use crate::PrefundWithdrawStakeQuoteErr;

pub trait StakeQuoteError {
    /// Indicates if this error is specific to the individual
    /// vote account involved and that the pool may be able
    /// to give a valid quote for a different vote account
    ///
    /// Examples:
    /// - vote account is not accepted by stake pool for DepositStake
    /// - validator stake account does not have enough liquidity for WithdrawStake
    fn is_vote_specific(&self) -> bool;
}

// we want to impl this trait for external types,
// so lets do a blanket for just `&`
/// Blanket for &
impl<T: StakeQuoteError> StakeQuoteError for &T {
    #[inline]
    fn is_vote_specific(&self) -> bool {
        (**self).is_vote_specific()
    }
}

impl StakeQuoteError for LidoError {
    #[inline]
    fn is_vote_specific(&self) -> bool {
        false
    }
}

impl StakeQuoteError for MarinadeError {
    #[inline]
    fn is_vote_specific(&self) -> bool {
        matches!(
            self,
            Self::WrongValidatorAccountOrIndex
            // WithdrawStake (not yet enabled)
            | Self::SelectedStakeAccountHasNotEnoughFunds
        )
    }
}

impl StakeQuoteError for ReserveError {
    #[inline]
    fn is_vote_specific(&self) -> bool {
        false
    }
}

impl StakeQuoteError for SplStakePoolError {
    #[inline]
    fn is_vote_specific(&self) -> bool {
        matches!(
            self,
            Self::ValidatorNotFound
                // DepositStake
                | Self::IncorrectDepositVoteAddress
                // WithdrawStake
                | Self::StakeLamportsNotEqualToMinimum
                | Self::IncorrectWithdrawVoteAddress
        )
    }
}

impl<E: StakeQuoteError> StakeQuoteError for PrefundWithdrawStakeQuoteErr<E> {
    #[inline]
    fn is_vote_specific(&self) -> bool {
        match self {
            Self::Pool(p) => p.is_vote_specific(),
            Self::Reserve(p) => p.is_vote_specific(),
        }
    }
}
