use generic_array_struct::generic_array_struct;
use sanctum_spl_stake_pool_core::{DepositSolQuoteArgs, SplStakePoolError, StakePool};

use crate::{DepositSolQuoter, DepositSolSufAccs, SplSolSufAccs, TokenQuote};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplDepositSolQuoter<'a> {
    pub stake_pool: &'a StakePool,
    pub curr_epoch: u64,
}

impl DepositSolQuoter for SplDepositSolQuoter<'_> {
    type Error = SplStakePoolError;

    #[inline]
    fn quote_deposit_sol(&self, lamports: u64) -> Result<TokenQuote, Self::Error> {
        self.stake_pool
            .quote_deposit_sol(
                lamports,
                DepositSolQuoteArgs {
                    // This automatically filters out permissioned pools with SOL deposit auth,
                    // since these pools will have Some(sol_deposit_auth) that
                    // does not match system program
                    depositor: [0; 32],
                    current_epoch: self.curr_epoch,
                },
            )
            .map(Into::into)
    }
}

impl DepositSolSufAccs for SplSolSufAccs<'_> {
    type Accs = SplDepositSolIxSuffixKeysOwned;
    type AccFlags = SplDepositSolIxSuffixAccsFlag;

    #[inline]
    fn suffix_accounts(&self) -> Self::Accs {
        SplDepositSolIxSuffixAccsBuilder::start()
            .with_stake_pool_program(*self.stake_pool_program)
            .with_stake_pool(*self.stake_pool_addr)
            .with_withdraw_auth(*self.withdraw_authority_program_address)
            .with_manager_fee(self.stake_pool.manager_fee_account)
            .with_reserve(self.stake_pool.reserve_stake)
            .build()
    }

    #[inline]
    fn suffix_is_signer(&self) -> Self::AccFlags {
        SPL_DEPOSIT_SOL_IX_SUFFIX_IS_SIGNER
    }

    #[inline]
    fn suffix_is_writable(&self) -> Self::AccFlags {
        SPL_DEPOSIT_SOL_IX_SUFFIX_IS_WRITER
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
pub struct SplDepositSolIxSuffixAccs<T> {
    pub stake_pool_program: T,
    pub stake_pool: T,
    pub withdraw_auth: T,
    pub reserve: T,
    pub manager_fee: T,
}
pub type SplDepositSolIxSuffixKeysOwned = SplDepositSolIxSuffixAccs<[u8; 32]>;
pub type SplDepositSolIxSuffixKeys<'a> = SplDepositSolIxSuffixAccs<&'a [u8; 32]>;
pub type SplDepositSolIxSuffixAccsFlag = SplDepositSolIxSuffixAccs<bool>;

pub const SPL_DEPOSIT_SOL_IX_SUFFIX_IS_WRITER: SplDepositSolIxSuffixAccsFlag =
    SplDepositSolIxSuffixAccs([false; SPL_DEPOSIT_SOL_IX_SUFFIX_ACCS_LEN])
        .const_with_stake_pool(true)
        .const_with_manager_fee(true)
        .const_with_reserve(true);

pub const SPL_DEPOSIT_SOL_IX_SUFFIX_IS_SIGNER: SplDepositSolIxSuffixAccsFlag =
    SplDepositSolIxSuffixAccs([false; SPL_DEPOSIT_SOL_IX_SUFFIX_ACCS_LEN]);

impl<T> SplDepositSolIxSuffixAccs<T> {
    #[inline]
    pub const fn new(arr: [T; SPL_DEPOSIT_SOL_IX_SUFFIX_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

impl<T> AsRef<[T]> for SplDepositSolIxSuffixAccs<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl SplDepositSolIxSuffixKeysOwned {
    #[inline]
    pub fn as_borrowed(&self) -> SplDepositSolIxSuffixKeys<'_> {
        SplDepositSolIxSuffixKeys::new(self.0.each_ref())
    }
}

impl SplDepositSolIxSuffixKeys<'_> {
    #[inline]
    pub fn into_owned(self) -> SplDepositSolIxSuffixKeysOwned {
        SplDepositSolIxSuffixKeysOwned::new(self.0.map(|pk| *pk))
    }
}
