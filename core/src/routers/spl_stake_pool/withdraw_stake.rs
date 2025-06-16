use core::{
    iter::{Chain, Map},
    slice,
};

use generic_array_struct::generic_array_struct;
use sanctum_spl_stake_pool_core::{
    SplStakePoolError, StakePool, ValidatorStakeInfo, WithdrawStakeQuoteArgs, MIN_ACTIVE_STAKE,
};

use crate::{
    ActiveStakeParams, StakeAccountLamports, WithdrawStakeQuote, WithdrawStakeQuoter,
    WithdrawStakeSufAccs, STAKE_PROGRAM, SYSTEM_PROGRAM, SYSVAR_CLOCK, TOKEN_PROGRAM,
};

#[derive(Debug, Clone, Copy)]
pub struct SplWithdrawStakeQuoter<'a> {
    pub stake_pool: &'a StakePool,
    pub curr_epoch: u64,
    pub validator_list: &'a [ValidatorStakeInfo],
}

impl SplWithdrawStakeQuoter<'_> {
    #[inline]
    pub fn find_max_validator(&self) -> Option<&ValidatorStakeInfo> {
        self.validator_list
            .iter()
            .max_by_key(|vsi| vsi.active_stake_lamports())
    }

    #[inline]
    pub fn find_validator_by_vote(&self, vote: &[u8; 32]) -> Option<&ValidatorStakeInfo> {
        self.validator_list
            .iter()
            .find(|vsi| vsi.vote_account_address() == vote)
    }
}

impl WithdrawStakeQuoter for SplWithdrawStakeQuoter<'_> {
    type Error = SplStakePoolError;

    #[inline]
    fn quote_withdraw_stake(
        &self,
        tokens: u64,
        vote: Option<&[u8; 32]>,
    ) -> Result<WithdrawStakeQuote, Self::Error> {
        let vsi = match (
            vote,
            self.stake_pool.preferred_withdraw_validator_vote_address,
        ) {
            (None, None) => self.find_max_validator(),
            (Some(v), None) => self.find_validator_by_vote(v),
            (v, Some(p)) => {
                if let Some(v) = v {
                    if *v != p {
                        return Err(SplStakePoolError::IncorrectWithdrawVoteAddress);
                    }
                }
                let preferred = self
                    .find_validator_by_vote(&p)
                    .ok_or(SplStakePoolError::ValidatorNotFound)?;
                if preferred.active_stake_lamports() <= MIN_ACTIVE_STAKE {
                    // preferred validator exhausted, users can withdraw from other validators
                    self.find_max_validator()
                } else {
                    Some(preferred)
                }
            }
        }
        .ok_or(SplStakePoolError::ValidatorNotFound)?;
        let quote = self.stake_pool.quote_withdraw_stake(
            tokens,
            WithdrawStakeQuoteArgs {
                current_epoch: self.curr_epoch,
            },
        )?;
        conv_quote(quote, vsi)
    }
}
// Q: why not just use `SplWithdrawStakeQuoter` but with a single elem slice for
// `validator_list`?
// A: need to handle preferred withdraw validator case a bit differently,
// since we dont have find_max_validator() here
//
/// [`SplWithdrawStakeQuoter`], but only for one of the validators in the pool
/// instead of all of them.
#[derive(Debug, Clone, Copy)]
pub struct SplWithdrawStakeValQuoter<'a> {
    pub stake_pool: &'a StakePool,
    pub curr_epoch: u64,
    pub validator: &'a ValidatorStakeInfo,
}

impl WithdrawStakeQuoter for SplWithdrawStakeValQuoter<'_> {
    type Error = SplStakePoolError;

    #[inline]
    fn quote_withdraw_stake(
        &self,
        tokens: u64,
        vote: Option<&[u8; 32]>,
    ) -> Result<WithdrawStakeQuote, Self::Error> {
        if let Some(v) = vote {
            if v != self.validator.vote_account_address() {
                return Err(SplStakePoolError::IncorrectWithdrawVoteAddress);
            }
        }
        let quote = self.stake_pool.quote_withdraw_stake(
            tokens,
            WithdrawStakeQuoteArgs {
                current_epoch: self.curr_epoch,
            },
        )?;
        conv_quote(quote, self.validator)
    }
}

pub type SplWithdrawStakeValQuoterSliceItr<'a, F> = Map<slice::Iter<'a, ValidatorStakeInfo>, F>;

pub type SplWithdrawStakeValQuoterItr<'a, F> =
    Chain<SplWithdrawStakeValQuoterSliceItr<'a, F>, SplWithdrawStakeValQuoterSliceItr<'a, F>>;

impl<'a> SplWithdrawStakeValQuoter<'a> {
    /// Returns an iterator of withdraw stake quoters for each validator on the list.
    ///
    /// Special cases if preferred withdraw validator is set:
    /// - if preferred withdraw validator is exhausted, an iterator over all other validators is returned
    /// - otherwise, a iterator yielding a single entry of the preferred withdraw validator is returned
    ///
    /// Returns Err if preferred withdraw validator is set but not on list.
    #[inline]
    pub fn all<'parent: 'a>(
        stake_pool: &'parent StakePool,
        validator_list: &'parent [ValidatorStakeInfo],
        curr_epoch: u64,
    ) -> Result<
        SplWithdrawStakeValQuoterItr<'a, impl Fn(&'a ValidatorStakeInfo) -> Self>,
        SplStakePoolError,
    > {
        // use 2 slices to accomodate preferred exhausted case
        let (s1, s2) = match stake_pool.preferred_withdraw_validator_vote_address {
            None => (validator_list, [].as_slice()),
            Some(p) => {
                let (i, preferred) = validator_list
                    .iter()
                    .enumerate()
                    .find(|(_i, vsi)| *vsi.vote_account_address() == p)
                    .ok_or(SplStakePoolError::ValidatorNotFound)?;
                if preferred.active_stake_lamports() <= MIN_ACTIVE_STAKE {
                    // preferred exhausted: return everything excluding preferred
                    // unchecked-index: i is in range [0, len-1],
                    // [i + 1..] will not panic even if i = len-1
                    (&validator_list[..i], &validator_list[i + 1..])
                } else {
                    // preferred not exhausted: return just preferred
                    (&validator_list[i..i + 1], [].as_slice())
                }
            }
        };
        let ctor = move |validator| Self {
            validator,
            stake_pool,
            curr_epoch,
        };
        Ok(s1.iter().map(ctor).chain(s2.iter().map(ctor)))
    }
}

fn conv_quote(
    sanctum_spl_stake_pool_core::WithdrawStakeQuote {
        tokens_in,
        lamports_staked,
        fee_amount,
    }: sanctum_spl_stake_pool_core::WithdrawStakeQuote,
    vsi: &ValidatorStakeInfo,
) -> Result<WithdrawStakeQuote, SplStakePoolError> {
    if lamports_staked > vsi.active_stake_lamports() {
        // StakeWithdrawalTooLarge
        return Err(SplStakePoolError::StakeLamportsNotEqualToMinimum);
    }
    Ok(WithdrawStakeQuote {
        inp: tokens_in,
        out: ActiveStakeParams {
            vote: *vsi.vote_account_address(),
            lamports: StakeAccountLamports {
                staked: lamports_staked,
                unstaked: 0,
            },
        },
        fee: fee_amount,
    })
}

#[derive(Debug, Clone, Copy)]
pub struct SplWithdrawStakeSufAccs<'a> {
    pub stake_pool_addr: &'a [u8; 32],
    pub stake_pool_program: &'a [u8; 32],
    pub stake_pool: &'a StakePool,

    /// Validator stake account
    pub validator_stake: [u8; 32],

    /// The stake withdraw authority PDA
    pub stake_withdraw_authority: &'a [u8; 32],
}

impl WithdrawStakeSufAccs for SplWithdrawStakeSufAccs<'_> {
    type Accs = SplWithdrawStakeIxSuffixKeysOwned;
    type AccFlags = SplWithdrawStakeIxSuffixAccsFlag;

    #[inline]
    fn suffix_accounts(&self) -> Self::Accs {
        SplWithdrawStakeIxSuffixAccsBuilder::start()
            .with_spl_stake_pool_program(*self.stake_pool_program)
            .with_spl_stake_pool(*self.stake_pool_addr)
            .with_validator_list(self.stake_pool.validator_list)
            .with_withdraw_authority(*self.stake_withdraw_authority)
            .with_stake_to_split(self.validator_stake)
            .with_manager_fee(self.stake_pool.manager_fee_account)
            .with_clock(SYSVAR_CLOCK)
            .with_token_program(TOKEN_PROGRAM)
            .with_stake_program(STAKE_PROGRAM)
            .with_system_program(SYSTEM_PROGRAM)
            .build()
    }

    #[inline]
    fn suffix_is_signer(&self) -> Self::AccFlags {
        SPL_WITHDRAW_STAKE_IX_SUFFIX_IS_SIGNER
    }

    #[inline]
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
    #[inline]
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
