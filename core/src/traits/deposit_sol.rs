use core::{error::Error, ops::Deref};

use crate::quote::TokenQuote;

pub trait DepositSolQuoter {
    type Error: Error;

    fn quote_deposit_sol(&self, lamports: u64) -> Result<TokenQuote, Self::Error>;
}

/// Blanket for refs
/// NB: this means we can only implement this trait for internal types
impl<R, T: DepositSolQuoter> DepositSolQuoter for R
where
    R: Deref<Target = T>,
{
    type Error = T::Error;

    #[inline]
    fn quote_deposit_sol(&self, lamports: u64) -> Result<TokenQuote, Self::Error> {
        self.deref().quote_deposit_sol(lamports)
    }
}

pub trait DepositSolSufAccs {
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
impl<R, T: DepositSolSufAccs> DepositSolSufAccs for R
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
