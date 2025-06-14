use core::{error::Error, fmt::Display, ops::Deref};

use sanctum_reserve_core::{FeeEnum, PoolBalance, ReserveError};
use sanctum_spl_stake_pool_core::STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS;

use crate::{
    reserves_has_enough_for_slumdog, slumdog_target_lamports, ActiveStakeParams, Prefund,
    StakeAccountLamports, StakeQuoteError, WithdrawStakeQuote,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrefundWithdrawStakeQuoteErr<E> {
    Reserve(ReserveError),
    Pool(E),
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
    ///    If `None`, the pool is allowed to choose any vote account
    fn quote_withdraw_stake(
        &self,
        tokens: u64,
        vote: Option<&[u8; 32]>,
    ) -> Result<WithdrawStakeQuote, Self::Error>;

    /// The default impl here assumes the program does not fund rent-exemption for the
    /// destination stake account that is split to during withdrawal.
    /// (get_withdraw_stake_quote()'s returned quote.out.unstaked = 0)
    fn quote_prefund_withdraw_stake(
        &self,
        tokens: u64,
        vote: Option<&[u8; 32]>,
        reserves_balance: &PoolBalance,
        reserves_fee: &FeeEnum,
    ) -> Result<Prefund<WithdrawStakeQuote>, PrefundWithdrawStakeQuoteErr<Self::Error>> {
        let WithdrawStakeQuote {
            inp,
            out: ActiveStakeParams { vote, lamports },
            fee,
        } = self
            .quote_withdraw_stake(tokens, vote)
            .map_err(PrefundWithdrawStakeQuoteErr::Pool)?;
        if !reserves_has_enough_for_slumdog(reserves_balance) {
            return Err(PrefundWithdrawStakeQuoteErr::Reserve(
                ReserveError::NotEnoughLiquidity,
            ));
        }
        // amount of active stake that will be split
        // from the withdrawn stake account to slumdog
        let prefund_fee = slumdog_target_lamports(reserves_balance, reserves_fee)
            .ok_or(PrefundWithdrawStakeQuoteErr::Reserve(
                ReserveError::InternalError,
            ))?
            .saturating_sub(STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS);
        Ok(Prefund {
            quote: WithdrawStakeQuote {
                inp,
                out: ActiveStakeParams {
                    vote,
                    lamports: StakeAccountLamports {
                        // return None if original quote does not give enough
                        // sol to repay prefund flash loan.
                        // TODO: even though this is a math error, it might be more
                        // helpful for consumers to return something like "WithdrawalTooSmall"
                        // instead but that will require adding it to sanctum-reserve-core
                        staked: lamports.total().checked_sub(prefund_fee).ok_or(
                            PrefundWithdrawStakeQuoteErr::Reserve(ReserveError::InternalError),
                        )?,
                        unstaked: STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS,
                    },
                },
                fee,
            },
            prefund_fee,
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
