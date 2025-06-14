use core::{error::Error, ops::Deref};

use crate::{ActiveStakeParams, DepositStakeQuote, StakeQuoteError};

pub trait DepositStakeQuoter {
    type Error: Error + StakeQuoteError;

    // pass `stake` by value since we're echoing it in return value anyway
    fn quote_deposit_stake(
        &self,
        stake: ActiveStakeParams,
    ) -> Result<DepositStakeQuote, Self::Error>;
}

/// Blanket for refs
/// NB: this means we can only implement this trait for internal types
impl<R, T: DepositStakeQuoter> DepositStakeQuoter for R
where
    R: Deref<Target = T>,
{
    type Error = T::Error;

    #[inline]
    fn quote_deposit_stake(
        &self,
        stake: ActiveStakeParams,
    ) -> Result<DepositStakeQuote, Self::Error> {
        self.deref().quote_deposit_stake(stake)
    }
}

pub trait DepositStakeSufAccs {
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
impl<R, T: DepositStakeSufAccs> DepositStakeSufAccs for R
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
