use crate::quote::DepositStakeQuote;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)
)]
pub struct StakeAccountLamports {
    pub staked: u64,
    pub unstaked: u64,
}

impl StakeAccountLamports {
    pub const fn total(&self) -> u64 {
        self.staked + self.unstaked
    }
}

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
