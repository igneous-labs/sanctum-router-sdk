use core::{error::Error, fmt::Display};

use sanctum_reserve_core::{FeeEnum, PoolBalance, ReserveError};

use crate::{
    DepositStakeQuote, DepositStakeQuoter, Prefund, PrefundWithdrawStakeQuoteErr,
    WithdrawStakeQuote, WithdrawStakeQuoter,
};

pub type QuotePrefundSwapViaStakeResult<W> =
    Result<(Prefund<WithdrawStakeQuote>, DepositStakeQuote), PrefundSwapViaStakeQuoteErr<W>>;

#[inline]
pub fn quote_prefund_swap_via_stake<W: WithdrawStakeQuoter>(
    w_itr: impl IntoIterator<Item = W>,
    d: impl DepositStakeQuoter,
    inp_tokens: u64,
    reserves_balance: &PoolBalance,
    reserves_fee: &FeeEnum,
) -> QuotePrefundSwapViaStakeResult<W::Error> {
    w_itr
        .into_iter()
        .filter_map(|w| {
            let wsq = match w.quote_prefund_withdraw_stake(
                inp_tokens,
                None,
                reserves_balance,
                reserves_fee,
            ) {
                Ok(q) => q,
                Err(e) => return Some(Err(e.into())),
            };
            let dsq = d.quote_deposit_stake(wsq.quote.out).ok()?;
            Some(Ok((wsq, dsq)))
        })
        .next()
        .map_or_else(|| Err(PrefundSwapViaStakeQuoteErr::NoMatch), |r| r)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrefundSwapViaStakeQuoteErr<W> {
    NoMatch,
    Reserve(ReserveError),
    Withdraw(W),
    // we dont catch DepositStake errs because the interface doesnt
    // differentiate between whether a quote was rejected because
    // the pool doesnt accept the validator or because of some
    // other irrecoverable failure
}

impl<W> From<PrefundWithdrawStakeQuoteErr<W>> for PrefundSwapViaStakeQuoteErr<W> {
    // cant make this a const fn due to generics
    #[inline]
    fn from(e: PrefundWithdrawStakeQuoteErr<W>) -> Self {
        match e {
            PrefundWithdrawStakeQuoteErr::Pool(e) => Self::Withdraw(e),
            PrefundWithdrawStakeQuoteErr::Reserve(e) => Self::Reserve(e),
        }
    }
}

impl<E: core::fmt::Debug> Display for PrefundSwapViaStakeQuoteErr<E> {
    // Display=Debug, since this is just a simple discriminated str enum
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<E: core::fmt::Debug> Error for PrefundSwapViaStakeQuoteErr<E> {}
