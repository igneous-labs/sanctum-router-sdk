use generic_array_struct::generic_array_struct;
use sanctum_spl_stake_pool_core::WithdrawStakeQuoteArgs;

use crate::{
    StakeAccountLamports, WithdrawStake, WithdrawStakeQuote, STAKE_PROGRAM, SYSTEM_PROGRAM,
    SYSVAR_CLOCK, TOKEN_PROGRAM,
};

use super::SplStakePoolWithdrawStakeRouter;

impl WithdrawStake for SplStakePoolWithdrawStakeRouter<'_> {
    type Accs = SplWithdrawStakeIxSuffixKeysOwned;
    type AccFlags = SplWithdrawStakeIxSuffixAccsFlag;

    /// Returned `quote.out.unstaked` = 0 because the program does not
    /// transfer rent-exempt lamports to split destination.
    /// Rent-exemption needs to be provided by someone else outside instruction.
    fn get_withdraw_stake_quote(&self, pool_tokens: u64) -> Option<WithdrawStakeQuote> {
        let sanctum_spl_stake_pool_core::WithdrawStakeQuote {
            tokens_in,
            lamports_staked,
            fee_amount,
        } = self
            .stake_pool
            .quote_withdraw_stake(
                pool_tokens,
                WithdrawStakeQuoteArgs {
                    current_epoch: self.current_epoch,
                },
            )
            .ok()?;

        if lamports_staked > self.max_split_lamports {
            // NotEnoughLiquidity
            return None;
        }

        Some(WithdrawStakeQuote {
            inp: tokens_in,
            out: StakeAccountLamports {
                staked: lamports_staked,
                unstaked: 0,
            },
            fee: fee_amount,
        })
    }

    fn suffix_accounts(&self) -> Self::Accs {
        SplWithdrawStakeIxSuffixAccsBuilder::start()
            .with_spl_stake_pool_program(*self.stake_pool_program)
            .with_spl_stake_pool(*self.stake_pool_addr)
            .with_validator_list(self.stake_pool.validator_list)
            .with_withdraw_authority(*self.withdraw_authority_program_address)
            .with_stake_to_split(self.validator_stake)
            .with_manager_fee(self.stake_pool.manager_fee_account)
            .with_clock(SYSVAR_CLOCK)
            .with_token_program(TOKEN_PROGRAM)
            .with_stake_program(STAKE_PROGRAM)
            .with_system_program(SYSTEM_PROGRAM)
            .build()
    }

    fn suffix_is_signer(&self) -> Self::AccFlags {
        SPL_WITHDRAW_STAKE_IX_SUFFIX_IS_SIGNER
    }

    fn suffix_is_writable(&self) -> Self::AccFlags {
        SPL_WITHDRAW_STAKE_IX_SUFFIX_IS_WRITER
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
pub struct SplWithdrawStakeIxSuffixAccs<T> {
    pub spl_stake_pool_program: T,
    pub spl_stake_pool: T,
    pub validator_list: T,
    pub withdraw_authority: T,
    pub stake_to_split: T,
    pub manager_fee: T,
    pub clock: T,
    pub token_program: T,
    pub stake_program: T,
    pub system_program: T,
}
pub type SplWithdrawStakeIxSuffixKeysOwned = SplWithdrawStakeIxSuffixAccs<[u8; 32]>;
pub type SplWithdrawStakeIxSuffixKeys<'a> = SplWithdrawStakeIxSuffixAccs<&'a [u8; 32]>;
pub type SplWithdrawStakeIxSuffixAccsFlag = SplWithdrawStakeIxSuffixAccs<bool>;

pub const SPL_WITHDRAW_STAKE_IX_SUFFIX_IS_WRITER: SplWithdrawStakeIxSuffixAccsFlag =
    SplWithdrawStakeIxSuffixAccs([false; SPL_WITHDRAW_STAKE_IX_SUFFIX_ACCS_LEN])
        .const_with_spl_stake_pool(true)
        .const_with_validator_list(true)
        .const_with_stake_to_split(true)
        .const_with_manager_fee(true);

pub const SPL_WITHDRAW_STAKE_IX_SUFFIX_IS_SIGNER: SplWithdrawStakeIxSuffixAccsFlag =
    SplWithdrawStakeIxSuffixAccs([false; SPL_WITHDRAW_STAKE_IX_SUFFIX_ACCS_LEN]);

impl<T> SplWithdrawStakeIxSuffixAccs<T> {
    #[inline]
    pub const fn new(arr: [T; SPL_WITHDRAW_STAKE_IX_SUFFIX_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

impl<T> AsRef<[T]> for SplWithdrawStakeIxSuffixAccs<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl SplWithdrawStakeIxSuffixKeysOwned {
    #[inline]
    pub fn as_borrowed(&self) -> SplWithdrawStakeIxSuffixKeys<'_> {
        SplWithdrawStakeIxSuffixKeys::new(self.0.each_ref())
    }
}

impl SplWithdrawStakeIxSuffixKeys<'_> {
    #[inline]
    pub fn into_owned(self) -> SplWithdrawStakeIxSuffixKeysOwned {
        SplWithdrawStakeIxSuffixKeysOwned::new(self.0.map(|pk| *pk))
    }
}
