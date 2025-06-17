use generic_array_struct::generic_array_struct;

use super::INSTRUCTION_IDX_PREFUND_WITHDRAW_STAKE;

#[generic_array_struct(builder pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct PrefundWithdrawStakePrefixAccs<T> {
    /// The withdraw authority of stake_account. Needs to be mutable and system account to receive slumlord flash loan.
    pub user: T,

    pub inp_token: T,

    /// The bridge stake account PDA thats withdrawn and given to the user.
    /// Might be long-lived, make sure the seed is not already in use.
    ///
    /// `seeds = ['bridge_stake', user.pubkey, bridge_stake_seed]`
    pub bridge_stake: T,

    pub inp_mint: T,

    /// The system account PDA that contains enough SOL
    /// to prefund 2 stake accounts for withdrawal.
    /// Someone must send SOL to here to initialize it.
    ///
    /// `seeds = ['prefunder']`
    pub prefunder: T,

    /// The slumdog stake account is split from bridge_stake upon stake withdraw
    /// and instant unstaked to repay slumlord's flash loan.
    /// Might be long-lived, but should be not in use as long as bridge_stake is not in use
    ///
    /// `create_with_seed(bridge_stake.pubkey, 'slumdog', stake_program)`
    pub slumdog_stake: T,

    pub unstake_program: T,
    pub unstake_pool: T,
    pub unstake_pool_sol_reserves: T,
    pub unstake_fee: T,

    /// Sanctum unstake pool stake account record for slumdog stake.
    /// PDA of sanctum unstake program.
    ///
    /// `seeds = [unstakePool.pubkey, slumdogStake.pubkey]`
    pub slumdog_stake_acc_record: T,

    pub unstake_protocol_fee: T,
    pub unstake_protocol_fee_dest: T,
    pub clock: T,
    pub stake_program: T,
    pub system_program: T,
}

pub type PrefundWithdrawStakePrefixKeysOwned = PrefundWithdrawStakePrefixAccs<[u8; 32]>;
pub type PrefundWithdrawStakePrefixKeys<'a> = PrefundWithdrawStakePrefixAccs<&'a [u8; 32]>;
pub type PrefundWithdrawStakePrefixAccsFlag = PrefundWithdrawStakePrefixAccs<bool>;

pub const PREFUND_WITHDRAW_STAKE_PREFIX_IS_WRITER: PrefundWithdrawStakePrefixAccsFlag =
    PrefundWithdrawStakePrefixAccs([true; PREFUND_WITHDRAW_STAKE_PREFIX_ACCS_LEN])
        .const_with_unstake_program(false)
        .const_with_unstake_fee(false)
        .const_with_unstake_protocol_fee(false)
        .const_with_clock(false)
        .const_with_stake_program(false)
        .const_with_system_program(false);

pub const PREFUND_WITHDRAW_STAKE_PREFIX_IS_SIGNER: PrefundWithdrawStakePrefixAccsFlag =
    PrefundWithdrawStakePrefixAccs([false; PREFUND_WITHDRAW_STAKE_PREFIX_ACCS_LEN])
        .const_with_user(true);

impl<T> PrefundWithdrawStakePrefixAccs<T> {
    pub const fn new(arr: [T; PREFUND_WITHDRAW_STAKE_PREFIX_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

impl PrefundWithdrawStakePrefixKeysOwned {
    #[inline]
    pub fn as_borrowed(&self) -> PrefundWithdrawStakePrefixKeys<'_> {
        PrefundWithdrawStakePrefixKeys::new(self.0.each_ref())
    }
}

impl PrefundWithdrawStakePrefixKeys<'_> {
    #[inline]
    pub fn into_owned(self) -> PrefundWithdrawStakePrefixKeysOwned {
        PrefundWithdrawStakePrefixKeysOwned::new(self.0.map(|pk| *pk))
    }
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PrefundWithdrawStakeIxData([u8; 13]);

impl PrefundWithdrawStakeIxData {
    #[inline]
    pub fn new(amount: u64, bridge_stake_seed: u32) -> Self {
        let mut buf = [0u8; 13];

        buf[0] = INSTRUCTION_IDX_PREFUND_WITHDRAW_STAKE;
        buf[1..9].copy_from_slice(&amount.to_le_bytes());
        buf[9..13].copy_from_slice(&bridge_stake_seed.to_le_bytes());

        Self(buf)
    }

    #[inline]
    pub const fn to_buf(&self) -> [u8; 13] {
        self.0
    }
}
