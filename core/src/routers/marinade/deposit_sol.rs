use generic_array_struct::generic_array_struct;
use sanctum_marinade_liquid_staking_core::DepositSolQuoteArgs;

use crate::{DepositSol, TokenQuote};

use super::MarinadeSolRouter;

impl DepositSol for MarinadeSolRouter<'_> {
    type Accs = MarinadeDepositSolIxSuffixKeysOwned;
    type AccFlags = MarinadeDepositSolIxSuffixAccsFlag;

    fn get_deposit_sol_quote(&self, lamports: u64) -> Option<TokenQuote> {
        let quote = self
            .state
            .quote_deposit_sol(
                lamports,
                DepositSolQuoteArgs {
                    msol_leg_balance: self.msol_leg_balance,
                },
            )
            .ok()?;
        Some(quote.into())
    }

    fn suffix_accounts(&self) -> Self::Accs {
        NewMarinadeDepositSolIxSuffixAccsBuilder::start()
            .with_marinade_program(sanctum_marinade_liquid_staking_core::MARINADE_STAKING_PROGRAM)
            .with_state(sanctum_marinade_liquid_staking_core::STATE_PUBKEY)
            .with_msol_mint_auth(sanctum_marinade_liquid_staking_core::MSOL_MINT_AUTHORITY_PUBKEY)
            .with_reserve(sanctum_marinade_liquid_staking_core::RESERVE_PUBKEY)
            .with_liq_pool_msol_leg(self.state.liq_pool.msol_leg)
            .with_liq_pool_msol_leg_auth(
                sanctum_marinade_liquid_staking_core::LIQ_POOL_MSOL_LEG_AUTHORITY_PUBKEY,
            )
            .with_liq_pool_sol_leg(sanctum_marinade_liquid_staking_core::LIQ_POOL_SOL_LEG_PUBKEY)
            .build()
    }

    fn suffix_is_signer(&self) -> Self::AccFlags {
        MARINADE_DEPOSIT_SOL_IX_SUFFIX_IS_SIGNER
    }

    fn suffix_is_writable(&self) -> Self::AccFlags {
        MARINADE_DEPOSIT_SOL_IX_SUFFIX_IS_WRITER
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
pub struct MarinadeDepositSolIxSuffixAccs<T> {
    pub marinade_program: T,
    pub state: T,
    pub liq_pool_sol_leg: T,
    pub liq_pool_msol_leg: T,
    pub liq_pool_msol_leg_auth: T,
    pub reserve: T,
    pub msol_mint_auth: T,
}
pub type MarinadeDepositSolIxSuffixKeysOwned = MarinadeDepositSolIxSuffixAccs<[u8; 32]>;
pub type MarinadeDepositSolIxSuffixKeys<'a> = MarinadeDepositSolIxSuffixAccs<&'a [u8; 32]>;
pub type MarinadeDepositSolIxSuffixAccsFlag = MarinadeDepositSolIxSuffixAccs<bool>;

pub const MARINADE_DEPOSIT_SOL_IX_SUFFIX_IS_WRITER: MarinadeDepositSolIxSuffixAccsFlag =
    MarinadeDepositSolIxSuffixAccs([false; MARINADE_DEPOSIT_SOL_IX_SUFFIX_ACCS_LEN])
        .const_with_state(true)
        .const_with_liq_pool_sol_leg(true)
        .const_with_liq_pool_msol_leg(true)
        .const_with_reserve(true);

pub const MARINADE_DEPOSIT_SOL_IX_SUFFIX_IS_SIGNER: MarinadeDepositSolIxSuffixAccsFlag =
    MarinadeDepositSolIxSuffixAccs([false; MARINADE_DEPOSIT_SOL_IX_SUFFIX_ACCS_LEN]);

impl<T> MarinadeDepositSolIxSuffixAccs<T> {
    #[inline]
    pub const fn new(arr: [T; MARINADE_DEPOSIT_SOL_IX_SUFFIX_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

impl<T> AsRef<[T]> for MarinadeDepositSolIxSuffixAccs<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl MarinadeDepositSolIxSuffixKeysOwned {
    #[inline]
    pub fn as_borrowed(&self) -> MarinadeDepositSolIxSuffixKeys<'_> {
        MarinadeDepositSolIxSuffixKeys::new(self.0.each_ref())
    }
}

impl MarinadeDepositSolIxSuffixKeys<'_> {
    #[inline]
    pub fn into_owned(&self) -> MarinadeDepositSolIxSuffixKeysOwned {
        MarinadeDepositSolIxSuffixKeysOwned::new(self.0.map(|pk| *pk))
    }
}
