use generic_array_struct::generic_array_struct;
use sanctum_spl_stake_pool_core::{DepositStakeQuoteArgs, StakeAccountLamports};

use crate::{
    DepositStake, DepositStakeQuote, STAKE_PROGRAM, SYSVAR_CLOCK, SYSVAR_STAKE_HISTORY,
    TOKEN_PROGRAM,
};

use super::SplStakePoolDepositStakeRouter;

impl DepositStake for SplStakePoolDepositStakeRouter<'_> {
    type Accs = SplDepositStakeIxSuffixKeysOwned;
    type AccFlags = SplDepositStakeIxSuffixAccsFlag;

    fn get_deposit_stake_quote(
        &self,
        crate::StakeAccountLamports { staked, unstaked }: crate::StakeAccountLamports,
    ) -> Option<DepositStakeQuote> {
        let quote = self
            .stake_pool
            .quote_deposit_stake(
                StakeAccountLamports { staked, unstaked },
                DepositStakeQuoteArgs {
                    validator_stake_info: *self.validator_stake_info,
                    validator: *self.validator_stake_info.vote_account_address(),
                    current_epoch: self.current_epoch,
                },
            )
            .ok()?;
        Some(quote.into())
    }

    fn suffix_accounts(&self) -> Self::Accs {
        SplDepositStakeIxSuffixAccsBuilder::start()
            .with_spl_stake_pool_program(*self.stake_pool_program)
            .with_spl_stake_pool(*self.stake_pool_addr)
            .with_deposit_authority(*self.deposit_authority_program_address)
            .with_withdraw_authority(*self.withdraw_authority_program_address)
            .with_validator_stake(self.validator_stake)
            .with_validator_list(self.stake_pool.validator_list)
            .with_reserve_stake(self.stake_pool.reserve_stake)
            .with_manager_fee(self.stake_pool.manager_fee_account)
            .with_clock(SYSVAR_CLOCK)
            .with_stake_history(SYSVAR_STAKE_HISTORY)
            .with_token_program(TOKEN_PROGRAM)
            .with_stake_program(STAKE_PROGRAM)
            .build()
    }

    fn suffix_is_signer(&self) -> Self::AccFlags {
        SPL_DEPOSIT_STAKE_IX_SUFFIX_IS_SIGNER
    }

    fn suffix_is_writable(&self) -> Self::AccFlags {
        SPL_DEPOSIT_STAKE_IX_SUFFIX_IS_WRITER
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
pub struct SplDepositStakeIxSuffixAccs<T> {
    pub spl_stake_pool_program: T,
    pub spl_stake_pool: T,
    pub validator_list: T,
    pub deposit_authority: T,
    pub withdraw_authority: T,
    pub validator_stake: T,
    pub reserve_stake: T,
    pub manager_fee: T,
    pub clock: T,
    pub stake_history: T,
    pub token_program: T,
    pub stake_program: T,
}
pub type SplDepositStakeIxSuffixKeysOwned = SplDepositStakeIxSuffixAccs<[u8; 32]>;
pub type SplDepositStakeIxSuffixKeys<'a> = SplDepositStakeIxSuffixAccs<&'a [u8; 32]>;
pub type SplDepositStakeIxSuffixAccsFlag = SplDepositStakeIxSuffixAccs<bool>;

pub const SPL_DEPOSIT_STAKE_IX_SUFFIX_IS_WRITER: SplDepositStakeIxSuffixAccsFlag =
    SplDepositStakeIxSuffixAccs([false; SPL_DEPOSIT_STAKE_IX_SUFFIX_ACCS_LEN])
        .const_with_spl_stake_pool(true)
        .const_with_validator_list(true)
        .const_with_validator_stake(true)
        .const_with_manager_fee(true)
        .const_with_reserve_stake(true);

pub const SPL_DEPOSIT_STAKE_IX_SUFFIX_IS_SIGNER: SplDepositStakeIxSuffixAccsFlag =
    SplDepositStakeIxSuffixAccs([false; SPL_DEPOSIT_STAKE_IX_SUFFIX_ACCS_LEN]);

impl<T> SplDepositStakeIxSuffixAccs<T> {
    #[inline]
    pub const fn new(arr: [T; SPL_DEPOSIT_STAKE_IX_SUFFIX_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

impl<T> AsRef<[T]> for SplDepositStakeIxSuffixAccs<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl SplDepositStakeIxSuffixKeysOwned {
    #[inline]
    pub fn as_borrowed(&self) -> SplDepositStakeIxSuffixKeys<'_> {
        SplDepositStakeIxSuffixKeys::new(self.0.each_ref())
    }
}

impl SplDepositStakeIxSuffixKeys<'_> {
    #[inline]
    pub fn into_owned(self) -> SplDepositStakeIxSuffixKeysOwned {
        SplDepositStakeIxSuffixKeysOwned::new(self.0.map(|pk| *pk))
    }
}
