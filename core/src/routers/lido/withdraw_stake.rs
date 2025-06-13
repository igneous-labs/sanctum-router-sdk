use generic_array_struct::generic_array_struct;
use solido_legacy_core::{max_withdraw_lamports, LidoError, Validator};

use crate::{
    ActiveStakeParams, StakeAccountLamports, WithdrawStakeQuote, WithdrawStakeQuoter,
    WithdrawStakeSufAccs, STAKE_PROGRAM, SYSTEM_PROGRAM, SYSVAR_CLOCK, TOKEN_PROGRAM,
};

#[derive(Debug, Clone)]
pub struct LidoWithdrawStakeQuoter<'a> {
    pub exchange_rate: &'a solido_legacy_core::ExchangeRate,
    pub largest_stake_vote: &'a [u8; 32],
    pub curr_epoch: u64,
    pub largest_stake_effective_stake_balance: u64,
}

impl<'a> LidoWithdrawStakeQuoter<'a> {
    #[inline]
    pub fn new(
        state: &'a solido_legacy_core::Lido,
        validator_list: &'a [Validator],
        curr_epoch: u64,
    ) -> Option<Self> {
        let largest_stake = validator_list
            .iter()
            .max_by_key(|v| v.effective_stake_balance())?;
        Some(Self {
            exchange_rate: &state.exchange_rate,
            largest_stake_vote: largest_stake.vote_account_address(),
            curr_epoch,
            largest_stake_effective_stake_balance: largest_stake.effective_stake_balance(),
        })
    }
}

impl WithdrawStakeQuoter for LidoWithdrawStakeQuoter<'_> {
    type Error = LidoError;

    #[inline]
    fn quote_withdraw_stake(
        &self,
        tokens: u64,
        vote: Option<&[u8; 32]>,
    ) -> Result<WithdrawStakeQuote, Self::Error> {
        if let Some(v) = vote {
            if v != self.largest_stake_vote {
                return Err(LidoError::ValidatorWithMoreStakeExists);
            }
        }
        if self.curr_epoch > self.exchange_rate.computed_in_epoch {
            return Err(LidoError::ExchangeRateNotUpdatedInThisEpoch);
        }
        let lamports_staked = self
            .exchange_rate
            .quote_withdraw_stake(tokens)
            .ok_or(LidoError::CalculationFailure)?;
        let max_withdraw_lamports =
            max_withdraw_lamports(self.largest_stake_effective_stake_balance)
                .ok_or(LidoError::CalculationFailure)?;
        if lamports_staked > max_withdraw_lamports {
            // StakeWithdrawalTooLarge
            return Err(LidoError::InvalidAmount);
        }
        Ok(WithdrawStakeQuote {
            inp: tokens,
            out: ActiveStakeParams {
                vote: *self.largest_stake_vote,
                lamports: StakeAccountLamports {
                    staked: lamports_staked,
                    unstaked: 0,
                },
            },
            fee: 0,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LidoWithdrawStakeSufAccs<'a> {
    pub validator_list_addr: &'a [u8; 32],
    pub largest_stake_vote: &'a [u8; 32],

    /// Lido program PDA of `largest_stake_vote`
    pub stake_to_split: [u8; 32],
}

impl WithdrawStakeSufAccs for LidoWithdrawStakeSufAccs<'_> {
    type Accs = LidoWithdrawStakeIxSuffixKeysOwned;
    type AccFlags = LidoWithdrawStakeIxSuffixAccsFlag;

    #[inline]
    fn suffix_accounts(&self) -> Self::Accs {
        LidoWithdrawStakeIxSuffixAccsBuilder::start()
            .with_lido_program(solido_legacy_core::PROGRAM_ID)
            .with_solido(solido_legacy_core::LIDO_STATE_ADDR)
            .with_stake_authority(solido_legacy_core::STAKE_AUTH_PDA)
            .with_validator_list(*self.validator_list_addr)
            .with_voter(*self.largest_stake_vote)
            .with_stake_to_split(self.stake_to_split)
            .with_clock(SYSVAR_CLOCK)
            .with_token_program(TOKEN_PROGRAM)
            .with_stake_program(STAKE_PROGRAM)
            .with_system_program(SYSTEM_PROGRAM)
            .build()
    }

    #[inline]
    fn suffix_is_signer(&self) -> Self::AccFlags {
        LIDO_WITHDRAW_STAKE_IX_SUFFIX_IS_SIGNER
    }

    #[inline]
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
    #[inline]
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
