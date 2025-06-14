use core::{error::Error, fmt::Display};

use sanctum_reserve_core::{FeeEnum, PoolBalance, ReserveError};

use crate::{
    DepositStakeQuote, DepositStakeQuoter, Prefund, PrefundWithdrawStakeQuoteErr, StakeQuoteError,
    WithdrawStakeQuote, WithdrawStakeQuoter,
};

pub type QuotePrefundSwapViaStakeResult<W, D> =
    Result<(Prefund<WithdrawStakeQuote>, DepositStakeQuote), PrefundSwapViaStakeQuoteErr<W, D>>;

#[inline]
pub fn quote_prefund_swap_via_stake<W: WithdrawStakeQuoter, D: DepositStakeQuoter>(
    w_itr: impl IntoIterator<Item = W>,
    d: D,
    inp_tokens: u64,
    reserves_balance: &PoolBalance,
    reserves_fee: &FeeEnum,
) -> QuotePrefundSwapViaStakeResult<W::Error, D::Error> {
    w_itr
        .into_iter()
        .filter_map(|w| {
            let wsq = match map_res(w.quote_prefund_withdraw_stake(
                inp_tokens,
                None,
                reserves_balance,
                reserves_fee,
            ))? {
                // stop iteration with err
                Err(e) => return Some(Err(e.into())),
                Ok(q) => q,
            };
            let dsq = match map_res(d.quote_deposit_stake(wsq.quote.out))? {
                // stop iteration with err
                Err(e) => return Some(Err(PrefundSwapViaStakeQuoteErr::DepositStake(e))),
                Ok(q) => q,
            };
            Some(Ok((wsq, dsq)))
        })
        .next()
        .map_or_else(|| Err(PrefundSwapViaStakeQuoteErr::NoMatch), |r| r)
}

/// Converts `Result<T, E>` to `Option<Result<T, E>>`
/// where `None` is returned if `Err(e) => e.is_vote_specific()`
/// while `Some` wraps the original result otherwise
#[inline]
fn map_res<T, E: StakeQuoteError>(res: Result<T, E>) -> Option<Result<T, E>> {
    res.map_or_else(
        |e| (!e.is_vote_specific()).then_some(Err(e)),
        // else is_vote_specific(), return None to continue iteration
        |q| Some(Ok(q)),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrefundSwapViaStakeQuoteErr<W, D> {
    NoMatch,
    Reserve(ReserveError),
    WithdrawStake(W),
    DepositStake(D),
}

impl<W, D> From<PrefundWithdrawStakeQuoteErr<W>> for PrefundSwapViaStakeQuoteErr<W, D> {
    // cant make this a const fn due to generics
    #[inline]
    fn from(e: PrefundWithdrawStakeQuoteErr<W>) -> Self {
        match e {
            PrefundWithdrawStakeQuoteErr::Pool(e) => Self::WithdrawStake(e),
            PrefundWithdrawStakeQuoteErr::Reserve(e) => Self::Reserve(e),
        }
    }
}

impl<W: core::fmt::Debug, D: core::fmt::Debug> Display for PrefundSwapViaStakeQuoteErr<W, D> {
    // Display=Debug, since this is just a simple discriminated str enum
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<W: core::fmt::Debug, D: core::fmt::Debug> Error for PrefundSwapViaStakeQuoteErr<W, D> {}
