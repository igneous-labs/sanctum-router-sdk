use generic_array_struct::generic_array_struct;
use sanctum_reserve_core::{
    quote_unstake, FeeEnum, PoolBalance, ProtocolFee, ReserveError, UnstakeQuote,
};

use crate::{
    slumdog_target_lamports, ActiveStakeParams, DepositStakeQuote, DepositStakeQuoter,
    DepositStakeSufAccs, STAKE_PROGRAM, SYSTEM_PROGRAM, SYSVAR_CLOCK, TOKEN_PROGRAM,
};

#[derive(Debug, Clone, Copy)]
pub struct ReserveDepositStakeQuoter<'a> {
    pub fee_account: &'a FeeEnum,
    pub protocol_fee_account: &'a ProtocolFee,
    pub pool_incoming_stake: u64,
    pub pool_sol_reserves: u64,
}

impl DepositStakeQuoter for ReserveDepositStakeQuoter<'_> {
    type Error = ReserveError;

    #[inline]
    fn quote_deposit_stake(
        &self,
        inp: ActiveStakeParams,
    ) -> Result<DepositStakeQuote, Self::Error> {
        self.quote_deposit_stake_inner(inp.lamports.total()).map(
            |UnstakeQuote {
                 lamports_to_unstaker,
                 fee,
                 ..
             }| DepositStakeQuote {
                inp,
                out: lamports_to_unstaker,
                fee: fee.total(),
            },
        )
    }
}

impl ReserveDepositStakeQuoter<'_> {
    #[inline]
    fn quote_deposit_stake_inner(
        &self,
        deposit_stake_total_lamports: u64,
    ) -> Result<UnstakeQuote, ReserveError> {
        // TODO: need to modify sanctum-reserve-core
        // to account for `ZERO_DATA_ACC_RENT_EXEMPT_LAMPORTS`
        quote_unstake(
            &self.pool_balance(),
            self.fee_account,
            &self.protocol_fee_account.fee_ratios(),
            deposit_stake_total_lamports,
            false,
        )
    }

    #[inline]
    fn pool_balance(&self) -> PoolBalance {
        PoolBalance {
            pool_incoming_stake: self.pool_incoming_stake,
            sol_reserves_lamports: self.pool_sol_reserves,
        }
    }

    #[inline]
    pub fn after_prefund(self) -> Result<Self, ReserveError> {
        let stake = slumdog_target_lamports(&self.pool_balance(), self.fee_account)
            .ok_or(ReserveError::InternalError)?;
        let quote = self.quote_deposit_stake_inner(stake)?;
        let Self {
            fee_account,
            protocol_fee_account,
            pool_incoming_stake,
            pool_sol_reserves,
        } = self;
        Ok(Self {
            fee_account,
            protocol_fee_account,
            // unchecked-arith: SOL supply is nowhere near u64::MAX
            pool_incoming_stake: pool_incoming_stake + quote.stake_account_lamports,
            // unchecked-arith: quote_deposit_stake_inner() passed means pool has enough liquidity
            pool_sol_reserves: pool_sol_reserves - quote.reserves_lamports_outflow(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ReserveDepositStakeSufAccs {
    pub stake_acc_record_addr: [u8; 32],
}

impl DepositStakeSufAccs for ReserveDepositStakeSufAccs {
    type Accs = ReserveDepositStakeIxSuffixKeysOwned;
    type AccFlags = ReserveDepositStakeIxSuffixAccsFlag;

    fn suffix_accounts(&self) -> Self::Accs {
        ReserveDepositStakeIxSuffixAccsBuilder::start()
            .with_reserve_program(sanctum_reserve_core::UNSTAKE_PROGRAM)
            .with_protocol_fee(sanctum_reserve_core::PROTOCOL_FEE)
            .with_pool_sol_reserves(sanctum_reserve_core::POOL_SOL_RESERVES)
            .with_reserve_fee(sanctum_reserve_core::FEE)
            .with_reserve_pool(sanctum_reserve_core::POOL)
            .with_protocol_fee_dest(sanctum_reserve_core::PROTOCOL_FEE_VAULT)
            .with_stake_acc_record(self.stake_acc_record_addr)
            .with_clock(SYSVAR_CLOCK)
            .with_system_program(SYSTEM_PROGRAM)
            .with_stake_program(STAKE_PROGRAM)
            .with_token_program(TOKEN_PROGRAM)
            .build()
    }

    fn suffix_is_signer(&self) -> Self::AccFlags {
        RESERVE_DEPOSIT_STAKE_IX_SUFFIX_IS_SIGNER
    }

    fn suffix_is_writable(&self) -> Self::AccFlags {
        RESERVE_DEPOSIT_STAKE_IX_SUFFIX_IS_WRITER
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
pub struct ReserveDepositStakeIxSuffixAccs<T> {
    pub reserve_program: T,
    pub reserve_pool: T,
    pub pool_sol_reserves: T,
    pub reserve_fee: T,
    pub stake_acc_record: T,
    pub protocol_fee: T,
    pub protocol_fee_dest: T,
    pub clock: T,
    pub stake_program: T,
    pub system_program: T,
    pub token_program: T,
}
pub type ReserveDepositStakeIxSuffixKeysOwned = ReserveDepositStakeIxSuffixAccs<[u8; 32]>;
pub type ReserveDepositStakeIxSuffixKeys<'a> = ReserveDepositStakeIxSuffixAccs<&'a [u8; 32]>;
pub type ReserveDepositStakeIxSuffixAccsFlag = ReserveDepositStakeIxSuffixAccs<bool>;

pub const RESERVE_DEPOSIT_STAKE_IX_SUFFIX_IS_WRITER: ReserveDepositStakeIxSuffixAccsFlag =
    ReserveDepositStakeIxSuffixAccs([false; RESERVE_DEPOSIT_STAKE_IX_SUFFIX_ACCS_LEN])
        .const_with_reserve_pool(true)
        .const_with_pool_sol_reserves(true)
        .const_with_stake_acc_record(true)
        .const_with_protocol_fee_dest(true);

pub const RESERVE_DEPOSIT_STAKE_IX_SUFFIX_IS_SIGNER: ReserveDepositStakeIxSuffixAccsFlag =
    ReserveDepositStakeIxSuffixAccs([false; RESERVE_DEPOSIT_STAKE_IX_SUFFIX_ACCS_LEN]);

impl<T> ReserveDepositStakeIxSuffixAccs<T> {
    #[inline]
    pub const fn new(arr: [T; RESERVE_DEPOSIT_STAKE_IX_SUFFIX_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

impl<T> AsRef<[T]> for ReserveDepositStakeIxSuffixAccs<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl ReserveDepositStakeIxSuffixKeysOwned {
    #[inline]
    pub fn as_borrowed(&self) -> ReserveDepositStakeIxSuffixKeys<'_> {
        ReserveDepositStakeIxSuffixKeys::new(self.0.each_ref())
    }
}

impl ReserveDepositStakeIxSuffixKeys<'_> {
    #[inline]
    pub fn into_owned(self) -> ReserveDepositStakeIxSuffixKeysOwned {
        ReserveDepositStakeIxSuffixKeysOwned::new(self.0.map(|pk| *pk))
    }
}
