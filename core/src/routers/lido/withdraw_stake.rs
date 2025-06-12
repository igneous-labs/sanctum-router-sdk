use generic_array_struct::generic_array_struct;
use solido_legacy_core::max_withdraw_lamports;

use crate::{
    StakeAccountLamports, WithdrawStake, WithdrawStakeQuote, STAKE_PROGRAM, SYSTEM_PROGRAM,
    SYSVAR_CLOCK, TOKEN_PROGRAM,
};

use super::LidoWithdrawStakeRouter;

impl WithdrawStake for LidoWithdrawStakeRouter<'_> {
    type Accs = LidoWithdrawStakeIxSuffixKeysOwned;

    type AccFlags = LidoWithdrawStakeIxSuffixAccsFlag;

    /// Returned `quote.out.unstaked` = 0 because the program only allocates and assigns
    /// stake accounts before splitting without transferring rent-exempt lamports.
    /// Rent-exemption needs to be provided by someone else outside instruction.
    fn get_withdraw_stake_quote(&self, pool_tokens: u64) -> Option<WithdrawStakeQuote> {
        if self.curr_epoch > self.state.exchange_rate.computed_in_epoch {
            return None;
        }
        let lamports_staked = self.state.exchange_rate.quote_withdraw_stake(pool_tokens)?;
        let max_withdraw_lamports = max_withdraw_lamports(self.validator_effective_stake_balance)?;
        if lamports_staked > max_withdraw_lamports {
            // NotEnoughLiquidity
            return None;
        }
        Some(WithdrawStakeQuote {
            inp: pool_tokens,
            out: StakeAccountLamports {
                staked: lamports_staked,
                unstaked: 0,
            },
            fee: 0,
        })
    }

    fn suffix_accounts(&self) -> Self::Accs {
        LidoWithdrawStakeIxSuffixAccsBuilder::start()
            .with_lido_program(solido_legacy_core::PROGRAM_ID)
            .with_solido(solido_legacy_core::LIDO_STATE_ADDR)
            .with_stake_authority(solido_legacy_core::STAKE_AUTH_PDA)
            .with_voter(*self.voter)
            .with_stake_to_split(self.stake_to_split)
            .with_validator_list(self.state.validator_list)
            .with_clock(SYSVAR_CLOCK)
            .with_token_program(TOKEN_PROGRAM)
            .with_stake_program(STAKE_PROGRAM)
            .with_system_program(SYSTEM_PROGRAM)
            .build()
    }

    fn suffix_is_signer(&self) -> Self::AccFlags {
        LIDO_WITHDRAW_STAKE_IX_SUFFIX_IS_SIGNER
    }

    fn suffix_is_writable(&self) -> Self::AccFlags {
        LIDO_WITHDRAW_STAKE_IX_SUFFIX_IS_WRITER
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
pub struct LidoWithdrawStakeIxSuffixAccs<T> {
    pub lido_program: T,
    pub solido: T,
    pub voter: T,
    pub stake_to_split: T,
    pub stake_authority: T,
    pub validator_list: T,
    pub clock: T,
    pub token_program: T,
    pub stake_program: T,
    pub system_program: T,
}
pub type LidoWithdrawStakeIxSuffixKeysOwned = LidoWithdrawStakeIxSuffixAccs<[u8; 32]>;
pub type LidoWithdrawStakeIxSuffixKeys<'a> = LidoWithdrawStakeIxSuffixAccs<&'a [u8; 32]>;
pub type LidoWithdrawStakeIxSuffixAccsFlag = LidoWithdrawStakeIxSuffixAccs<bool>;

pub const LIDO_WITHDRAW_STAKE_IX_SUFFIX_IS_WRITER: LidoWithdrawStakeIxSuffixAccsFlag =
    LidoWithdrawStakeIxSuffixAccs([false; LIDO_WITHDRAW_STAKE_IX_SUFFIX_ACCS_LEN])
        .const_with_solido(true)
        .const_with_validator_list(true)
        .const_with_stake_to_split(true);

pub const LIDO_WITHDRAW_STAKE_IX_SUFFIX_IS_SIGNER: LidoWithdrawStakeIxSuffixAccsFlag =
    LidoWithdrawStakeIxSuffixAccs([false; LIDO_WITHDRAW_STAKE_IX_SUFFIX_ACCS_LEN]);

impl<T> LidoWithdrawStakeIxSuffixAccs<T> {
    #[inline]
    pub const fn new(arr: [T; LIDO_WITHDRAW_STAKE_IX_SUFFIX_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

impl<T> AsRef<[T]> for LidoWithdrawStakeIxSuffixAccs<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl LidoWithdrawStakeIxSuffixKeysOwned {
    #[inline]
    pub fn as_borrowed(&self) -> LidoWithdrawStakeIxSuffixKeys<'_> {
        LidoWithdrawStakeIxSuffixKeys::new(self.0.each_ref())
    }
}

impl LidoWithdrawStakeIxSuffixKeys<'_> {
    #[inline]
    pub fn into_owned(self) -> LidoWithdrawStakeIxSuffixKeysOwned {
        LidoWithdrawStakeIxSuffixKeysOwned::new(self.0.map(|pk| *pk))
    }
}
