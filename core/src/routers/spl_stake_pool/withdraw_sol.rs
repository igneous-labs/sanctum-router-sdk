use generic_array_struct::generic_array_struct;
use sanctum_spl_stake_pool_core::{SplStakePoolError, StakePool, WithdrawSolQuoteArgs};

use crate::{
    SplSolSufAccs, TokenQuote, WithdrawSolQuoter, WithdrawSolSufAccs, STAKE_PROGRAM, SYSVAR_CLOCK,
    SYSVAR_STAKE_HISTORY,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplWithdrawSolQuoter<'a> {
    pub stake_pool: &'a StakePool,
    pub reserve_stake_lamports: u64,
    pub curr_epoch: u64,
}

impl WithdrawSolQuoter for SplWithdrawSolQuoter<'_> {
    type Error = SplStakePoolError;

    #[inline]
    fn quote_withdraw_sol(&self, lamports: u64) -> Result<TokenQuote, Self::Error> {
        self.stake_pool
            .quote_withdraw_sol(
                lamports,
                WithdrawSolQuoteArgs {
                    current_epoch: self.curr_epoch,
                    reserve_stake_lamports: self.reserve_stake_lamports,
                },
            )
            .map(Into::into)
    }
}

impl WithdrawSolSufAccs for SplSolSufAccs<'_> {
    type Accs = SplWithdrawSolIxSuffixKeysOwned;
    type AccFlags = SplWithdrawSolIxSuffixAccsFlag;

    #[inline]
    fn suffix_accounts(&self) -> Self::Accs {
        SplWithdrawSolIxSuffixAccsBuilder::start()
            .with_spl_stake_pool_program(*self.stake_pool_program)
            .with_spl_stake_pool(*self.stake_pool_addr)
            .with_withdraw_authority(*self.withdraw_authority_program_address)
            .with_reserve_stake(self.stake_pool.reserve_stake)
            .with_manager_fee(self.stake_pool.manager_fee_account)
            .with_clock(SYSVAR_CLOCK)
            .with_stake_history(SYSVAR_STAKE_HISTORY)
            .with_stake_program(STAKE_PROGRAM)
            .with_token_program(self.stake_pool.token_program_id)
            .build()
    }

    #[inline]
    fn suffix_is_signer(&self) -> Self::AccFlags {
        SPL_WITHDRAW_SOL_IX_SUFFIX_IS_SIGNER
    }

    #[inline]
    fn suffix_is_writable(&self) -> Self::AccFlags {
        SPL_WITHDRAW_SOL_IX_SUFFIX_IS_WRITER
    }
}

#[generic_array_struct(builder pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct SplWithdrawSolIxSuffixAccs<T> {
    pub spl_stake_pool_program: T,
    pub spl_stake_pool: T,
    pub withdraw_authority: T,
    pub reserve_stake: T,
    pub manager_fee: T,
    pub clock: T,
    pub stake_history: T,
    pub stake_program: T,
    /// possible duplicate to account for token-22 stake pools
    pub token_program: T,
}
pub type SplWithdrawSolIxSuffixKeysOwned = SplWithdrawSolIxSuffixAccs<[u8; 32]>;
pub type SplWithdrawSolIxSuffixKeys<'a> = SplWithdrawSolIxSuffixAccs<&'a [u8; 32]>;
pub type SplWithdrawSolIxSuffixAccsFlag = SplWithdrawSolIxSuffixAccs<bool>;

pub const SPL_WITHDRAW_SOL_IX_SUFFIX_IS_WRITER: SplWithdrawSolIxSuffixAccsFlag =
    SplWithdrawSolIxSuffixAccs([false; SPL_WITHDRAW_SOL_IX_SUFFIX_ACCS_LEN])
        .const_with_spl_stake_pool(true)
        .const_with_reserve_stake(true)
        .const_with_manager_fee(true);

pub const SPL_WITHDRAW_SOL_IX_SUFFIX_IS_SIGNER: SplWithdrawSolIxSuffixAccsFlag =
    SplWithdrawSolIxSuffixAccs([false; SPL_WITHDRAW_SOL_IX_SUFFIX_ACCS_LEN]);

impl<T> SplWithdrawSolIxSuffixAccs<T> {
    #[inline]
    pub const fn new(arr: [T; SPL_WITHDRAW_SOL_IX_SUFFIX_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

impl<T> AsRef<[T]> for SplWithdrawSolIxSuffixAccs<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl SplWithdrawSolIxSuffixKeysOwned {
    #[inline]
    pub fn as_borrowed(&self) -> SplWithdrawSolIxSuffixKeys<'_> {
        SplWithdrawSolIxSuffixKeys::new(self.0.each_ref())
    }
}

impl SplWithdrawSolIxSuffixKeys<'_> {
    #[inline]
    pub fn into_owned(&self) -> SplWithdrawSolIxSuffixKeysOwned {
        SplWithdrawSolIxSuffixKeysOwned::new(self.0.map(|pk| *pk))
    }
}
