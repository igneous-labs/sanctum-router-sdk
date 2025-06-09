use generic_array_struct::generic_array_struct;
use sanctum_marinade_liquid_staking_core::{
    DepositStakeQuoteArgs, StakeAccountLamports, MARINADE_STAKING_PROGRAM,
    MSOL_MINT_AUTHORITY_PUBKEY, STATE_PUBKEY,
};

use crate::{
    DepositStake, STAKE_PROGRAM, SYSTEM_PROGRAM, SYSVAR_CLOCK, SYSVAR_RENT, TOKEN_PROGRAM,
};

use super::MarinadeStakeRouter;

impl DepositStake for MarinadeStakeRouter<'_> {
    type Accs = MarinadeDepositStakeIxSuffixKeysOwned;
    type AccFlags = MarinadeDepositStakeIxSuffixAccsFlag;

    fn get_deposit_stake_quote(
        &self,
        crate::traits::StakeAccountLamports { staked, unstaked }: crate::traits::StakeAccountLamports,
    ) -> Option<crate::DepositStakeQuote> {
        let quote = self
            .state
            .quote_deposit_stake(
                StakeAccountLamports { staked, unstaked },
                DepositStakeQuoteArgs {
                    msol_leg_balance: self.msol_leg_balance,
                },
            )
            .ok()?;
        Some(quote.into())
    }

    fn suffix_accounts(&self) -> Self::Accs {
        MarinadeDepositStakeIxSuffixAccsBuilder::start()
            .with_marinade_program(MARINADE_STAKING_PROGRAM)
            .with_marinade_state(STATE_PUBKEY)
            .with_clock(SYSVAR_CLOCK)
            .with_rent(SYSVAR_RENT)
            .with_stake_program(STAKE_PROGRAM)
            .with_system_program(SYSTEM_PROGRAM)
            .with_token_program(TOKEN_PROGRAM)
            .with_msol_mint_auth(MSOL_MINT_AUTHORITY_PUBKEY)
            .with_stake_list(self.state.stake_system.stake_list.account)
            .with_validator_list(self.state.validator_system.validator_list.account)
            .with_duplication_flag(self.duplication_flag)
            .build()
    }

    fn suffix_is_signer(&self) -> Self::AccFlags {
        MARINADE_DEPOSIT_STAKE_IX_SUFFIX_IS_SIGNER
    }

    fn suffix_is_writable(&self) -> Self::AccFlags {
        MARINADE_DEPOSIT_STAKE_IX_SUFFIX_IS_WRITER
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
pub struct MarinadeDepositStakeIxSuffixAccs<T> {
    pub marinade_program: T,
    pub marinade_state: T,
    pub validator_list: T,
    pub stake_list: T,
    pub duplication_flag: T,
    pub msol_mint_auth: T,
    pub clock: T,
    pub rent: T,
    pub system_program: T,
    pub token_program: T,
    pub stake_program: T,
}
pub type MarinadeDepositStakeIxSuffixKeysOwned = MarinadeDepositStakeIxSuffixAccs<[u8; 32]>;
pub type MarinadeDepositStakeIxSuffixKeys<'a> = MarinadeDepositStakeIxSuffixAccs<&'a [u8; 32]>;
pub type MarinadeDepositStakeIxSuffixAccsFlag = MarinadeDepositStakeIxSuffixAccs<bool>;

pub const MARINADE_DEPOSIT_STAKE_IX_SUFFIX_IS_WRITER: MarinadeDepositStakeIxSuffixAccsFlag =
    MarinadeDepositStakeIxSuffixAccs([false; MARINADE_DEPOSIT_STAKE_IX_SUFFIX_ACCS_LEN])
        .const_with_stake_list(true)
        .const_with_validator_list(true)
        .const_with_marinade_state(true)
        .const_with_duplication_flag(true);

pub const MARINADE_DEPOSIT_STAKE_IX_SUFFIX_IS_SIGNER: MarinadeDepositStakeIxSuffixAccsFlag =
    MarinadeDepositStakeIxSuffixAccs([false; MARINADE_DEPOSIT_STAKE_IX_SUFFIX_ACCS_LEN]);

impl<T> MarinadeDepositStakeIxSuffixAccs<T> {
    #[inline]
    pub const fn new(arr: [T; MARINADE_DEPOSIT_STAKE_IX_SUFFIX_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

impl<T> AsRef<[T]> for MarinadeDepositStakeIxSuffixAccs<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl MarinadeDepositStakeIxSuffixKeysOwned {
    #[inline]
    pub fn as_borrowed(&self) -> MarinadeDepositStakeIxSuffixKeys<'_> {
        MarinadeDepositStakeIxSuffixKeys::new(self.0.each_ref())
    }
}

impl MarinadeDepositStakeIxSuffixKeys<'_> {
    #[inline]
    pub fn into_owned(&self) -> MarinadeDepositStakeIxSuffixKeysOwned {
        MarinadeDepositStakeIxSuffixKeysOwned::new(self.0.map(|pk| *pk))
    }
}
