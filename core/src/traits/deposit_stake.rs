use crate::{quote::DepositStakeQuote, StakeAccountLamports};

pub trait DepositStake {
    type Accs: AsRef<[[u8; 32]]>;
    type AccFlags: AsRef<[bool]>;

    fn get_deposit_stake_quote(
        &self,
        stake_account_lamports: StakeAccountLamports,
    ) -> Option<DepositStakeQuote>;

    /// Returned array must have `length = self.suffix_accounts_len()`
    fn suffix_accounts(&self) -> Self::Accs;

    /// Returned array must have `length = self.suffix_accounts_len()`
    fn suffix_is_signer(&self) -> Self::AccFlags;

    /// Returned array must have `length = self.suffix_accounts_len()`
    fn suffix_is_writable(&self) -> Self::AccFlags;
}
