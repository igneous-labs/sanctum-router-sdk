use generic_array_struct::generic_array_struct;

use super::INSTRUCTION_IDX_PREFUND_SWAP_VIA_STAKE;

#[generic_array_struct(builder pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
pub struct PrefundSwapViaStakePrefixAccs<T> {
    pub user: T,
    pub inp_token: T,
    pub out_token: T,
    pub bridge_stake: T,
    pub out_fee_token: T,
    pub inp_mint: T,
    pub out_mint: T,
    pub prefunder: T,
    pub slumdog_stake: T,
    pub unstake_program: T,
    pub unstake_pool: T,
    pub unstake_pool_sol_reserves: T,
    pub unstake_fee: T,
    pub slumdog_stake_acc_record: T,
    pub unstake_protocol_fee: T,
    pub unstake_protocol_fee_dest: T,
    pub clock: T,
    pub stake_program: T,
    pub system_program: T,
}

pub type PrefundSwapViaStakePrefixKeysOwned = PrefundSwapViaStakePrefixAccs<[u8; 32]>;
pub type PrefundSwapViaStakePrefixKeys<'a> = PrefundSwapViaStakePrefixAccs<&'a [u8; 32]>;
pub type PrefundSwapViaStakePrefixAccsFlag = PrefundSwapViaStakePrefixAccs<bool>;

pub const PREFUND_SWAP_VIA_STAKE_PREFIX_IS_WRITER_NON_WSOL_OUT: PrefundSwapViaStakePrefixAccsFlag =
    PrefundSwapViaStakePrefixAccs([true; PREFUND_SWAP_VIA_STAKE_PREFIX_ACCS_LEN])
        .const_with_unstake_program(false)
        .const_with_unstake_fee(false)
        .const_with_unstake_protocol_fee(false)
        .const_with_clock(false)
        .const_with_stake_program(false)
        .const_with_system_program(false);

/// If output mint is wsol, it must be set to readonly
pub const PREFUND_SWAP_VIA_STAKE_PREFIX_IS_WRITER_WSOL_OUT: PrefundSwapViaStakePrefixAccsFlag =
    PREFUND_SWAP_VIA_STAKE_PREFIX_IS_WRITER_NON_WSOL_OUT.const_with_out_mint(false);

pub const PREFUND_SWAP_VIA_STAKE_PREFIX_IS_SIGNER: PrefundSwapViaStakePrefixAccsFlag =
    PrefundSwapViaStakePrefixAccs([false; PREFUND_SWAP_VIA_STAKE_PREFIX_ACCS_LEN])
        .const_with_user(true);

impl<T> PrefundSwapViaStakePrefixAccs<T> {
    #[inline]
    pub const fn new(arr: [T; PREFUND_SWAP_VIA_STAKE_PREFIX_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

impl PrefundSwapViaStakePrefixKeysOwned {
    #[inline]
    pub fn as_borrowed(&self) -> PrefundSwapViaStakePrefixKeys<'_> {
        PrefundSwapViaStakePrefixKeys::new(self.0.each_ref())
    }
}

impl PrefundSwapViaStakePrefixKeys<'_> {
    #[inline]
    pub fn into_owned(self) -> PrefundSwapViaStakePrefixKeysOwned {
        PrefundSwapViaStakePrefixKeysOwned::new(self.0.map(|pk| *pk))
    }
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PrefundSwapViaStakeIxData([u8; 13]);

impl PrefundSwapViaStakeIxData {
    #[inline]
    pub fn new(amount: u64, bridge_stake_seed: u32) -> Self {
        let mut buf = [0u8; 13];

        buf[0] = INSTRUCTION_IDX_PREFUND_SWAP_VIA_STAKE;
        buf[1..9].copy_from_slice(&amount.to_le_bytes());
        buf[9..13].copy_from_slice(&bridge_stake_seed.to_le_bytes());

        Self(buf)
    }

    #[inline]
    pub const fn to_buf(&self) -> [u8; 13] {
        self.0
    }
}
