use core::{error::Error, fmt::Display, ops::Deref};

use sanctum_reserve_core::{FeeEnum, PoolUnstakeParams, ReserveError};
use sanctum_spl_stake_pool_core::STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS;

use crate::{
    reserves_has_enough_for_slumdog, ActiveStakeParams, Prefund, StakeAccountLamports,
    StakeQuoteError, WithdrawStakeQuote, SLUMDOG_TARGET_LAMPORTS,
    STAKE_PROG_MIN_DELEGATION_LAMPORTS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrefundWithdrawStakeQuoteErr<E> {
    Reserve(ReserveError),
    Pool(E),

    /// The stake account to withdraw is too small
    /// to even pay for the prefund flash loan
    TooSmall,
}

impl<E: core::fmt::Debug> Display for PrefundWithdrawStakeQuoteErr<E> {
    // Display=Debug, since this is just a simple discriminated str enum
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<E: core::fmt::Debug> Error for PrefundWithdrawStakeQuoteErr<E> {}

pub trait WithdrawStakeQuoter {
    type Error: Error + StakeQuoteError;

    /// # Params
    /// - `tokens` LST tokens to redeem to stake, in atomics
    /// - `vote` vote account for the withdrawn stake to be delegated to.
    ///   If `None`, the pool is allowed to choose any vote account
    fn quote_withdraw_stake(
        &self,
        tokens: u64,
        vote: Option<&[u8; 32]>,
    ) -> Result<WithdrawStakeQuote, Self::Error>;

    /// The default impl here assumes the program does not fund rent-exemption for the
    /// destination stake account that is split to during withdrawal.
    /// (get_withdraw_stake_quote()'s returned quote.out.unstaked = 0).
    ///
    /// This is currently true for all supported stake pools:
    /// - Spl
    /// - Lido
    fn quote_prefund_withdraw_stake(
        &self,
        tokens: u64,
        vote: Option<&[u8; 32]>,
        reserves_unstake_params: &PoolUnstakeParams,
        reserves_fee: &FeeEnum,
    ) -> Result<Prefund<WithdrawStakeQuote>, PrefundWithdrawStakeQuoteErr<Self::Error>> {
        // amount of active stake that will be split
        // from the withdrawn stake account to slumdog
        const PREFUND_FEE: u64 = SLUMDOG_TARGET_LAMPORTS - STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS;

        let WithdrawStakeQuote {
            inp,
            out: ActiveStakeParams { vote, lamports },
            fee,
        } = self
            .quote_withdraw_stake(tokens, vote)
            .map_err(PrefundWithdrawStakeQuoteErr::Pool)?;
        reserves_has_enough_for_slumdog(reserves_unstake_params, reserves_fee)
            .map_err(PrefundWithdrawStakeQuoteErr::Reserve)?;

        // doc-comment precondition
        assert!(lamports.unstaked == 0);

        let lamports_staked = lamports
            .staked
            .checked_sub(PREFUND_FEE)
            .ok_or(PrefundWithdrawStakeQuoteErr::TooSmall)?;
        if lamports_staked < STAKE_PROG_MIN_DELEGATION_LAMPORTS {
            return Err(PrefundWithdrawStakeQuoteErr::TooSmall);
        }

        Ok(Prefund {
            quote: WithdrawStakeQuote {
                inp,
                out: ActiveStakeParams {
                    vote,
                    lamports: StakeAccountLamports {
                        staked: lamports_staked,
                        unstaked: STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS,
                    },
                },
                fee,
            },
            prefund_fee: PREFUND_FEE,
        })
    }
}

/// Blanket for refs
/// NB: this means we can only implement this trait for internal types
impl<R, T: WithdrawStakeQuoter> WithdrawStakeQuoter for R
where
    R: Deref<Target = T>,
{
    type Error = T::Error;

    #[inline]
    fn quote_withdraw_stake(
        &self,
        tokens: u64,
        vote: Option<&[u8; 32]>,
    ) -> Result<WithdrawStakeQuote, Self::Error> {
        self.deref().quote_withdraw_stake(tokens, vote)
    }
}

pub trait WithdrawStakeSufAccs {
    type Accs: AsRef<[[u8; 32]]>;
    type AccFlags: AsRef<[bool]>;

    /// Returned array must have `length = self.suffix_accounts_len()`
    fn suffix_accounts(&self) -> Self::Accs;

    /// Returned array must have `length = self.suffix_accounts_len()`
    fn suffix_is_signer(&self) -> Self::AccFlags;

    /// Returned array must have `length = self.suffix_accounts_len()`
    fn suffix_is_writable(&self) -> Self::AccFlags;
}

/// Blanket for refs
/// NB: this means we can only implement this trait for internal types
impl<R, T: WithdrawStakeSufAccs> WithdrawStakeSufAccs for R
where
    R: Deref<Target = T>,
{
    type Accs = T::Accs;
    type AccFlags = T::AccFlags;

    #[inline]
    fn suffix_accounts(&self) -> Self::Accs {
        self.deref().suffix_accounts()
    }

    #[inline]
    fn suffix_is_signer(&self) -> Self::AccFlags {
        self.deref().suffix_is_signer()
    }

    #[inline]
    fn suffix_is_writable(&self) -> Self::AccFlags {
        self.deref().suffix_is_writable()
    }
}
