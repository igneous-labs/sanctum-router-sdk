use generic_array_struct::generic_array_struct;
use sanctum_spl_stake_pool_core::{
    DepositStakeQuoteArgs, SplStakePoolError, StakePool, ValidatorStakeInfo,
};

use crate::{
    ActiveStakeParams, DepositStakeQuote, DepositStakeQuoter, DepositStakeSufAccs, STAKE_PROGRAM,
    SYSVAR_CLOCK, SYSVAR_STAKE_HISTORY, TOKEN_PROGRAM,
};

#[derive(Debug, Clone)]
pub struct SplDepositStakeQuoter<'a> {
    pub stake_pool: &'a StakePool,
    pub current_epoch: u64,
    pub validator_list: &'a [ValidatorStakeInfo],

    /// The pool's default stake deposit authority PDA
    pub default_stake_deposit_authority: &'a [u8; 32],
}

impl DepositStakeQuoter for SplDepositStakeQuoter<'_> {
    type Error = SplStakePoolError;

    #[inline]
    fn quote_deposit_stake(
        &self,
        stake: ActiveStakeParams,
    ) -> Result<DepositStakeQuote, Self::Error> {
        // we do not handle private pools with custom deposit auths
        if self.stake_pool.stake_deposit_authority != *self.default_stake_deposit_authority {
            return Err(SplStakePoolError::InvalidStakeDepositAuthority);
        }

        let vsi = self
            .validator_list
            .iter()
            .find(|vsi| *vsi.vote_account_address() == stake.vote)
            .ok_or(SplStakePoolError::ValidatorNotFound)?;
        // .quote_deposit_stake() ensures preferred validator matches if set
        self.stake_pool
            .quote_deposit_stake(
                sanctum_spl_stake_pool_core::StakeAccountLamports {
                    staked: stake.lamports.staked,
                    unstaked: stake.lamports.unstaked,
                },
                &DepositStakeQuoteArgs::new(vsi, self.current_epoch),
            )
            .map(
                |sanctum_spl_stake_pool_core::DepositStakeQuote {
                     tokens_out,
                     referral_fee,
                     manager_fee,
                     ..
                 }| {
                    DepositStakeQuote {
                        inp: stake,
                        // we set referral destination = out token acc, so the user gets the referral fee
                        out: tokens_out + referral_fee,
                        fee: manager_fee,
                    }
                },
            )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SplDepositStakeSufAccs<'a> {
    pub stake_pool_addr: &'a [u8; 32],
    pub stake_pool_program: &'a [u8; 32],
    pub stake_pool: &'a StakePool,

    /// Validator stake account
    pub validator_stake: [u8; 32],

    /// The stake deposit authority PDA
    pub stake_deposit_authority: &'a [u8; 32],

    /// The stake withdraw authority PDA
    pub stake_withdraw_authority: &'a [u8; 32],
}

impl DepositStakeSufAccs for SplDepositStakeSufAccs<'_> {
    type Accs = SplDepositStakeIxSuffixKeysOwned;
    type AccFlags = SplDepositStakeIxSuffixAccsFlag;

    #[inline]
    fn suffix_accounts(&self) -> Self::Accs {
        SplDepositStakeIxSuffixAccsBuilder::start()
            .with_spl_stake_pool_program(*self.stake_pool_program)
            .with_spl_stake_pool(*self.stake_pool_addr)
            .with_deposit_authority(*self.stake_deposit_authority)
            .with_withdraw_authority(*self.stake_withdraw_authority)
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

    #[inline]
    fn suffix_is_signer(&self) -> Self::AccFlags {
        SPL_DEPOSIT_STAKE_IX_SUFFIX_IS_SIGNER
    }

    #[inline]
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
