use generic_array_struct::generic_array_struct;

use crate::{SOL_BRIDGE_OUT, SYSTEM_PROGRAM, WSOL_BRIDGE_IN};

use super::INSTRUCTION_IDX_STAKE_WRAPPED_SOL;

#[generic_array_struct(pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct StakeWrappedSolPrefixAccs<T> {
    pub user: T,
    pub inp_wsol: T,
    pub out_token: T,
    pub wsol_bridge_in: T,
    pub sol_bridge_out: T,
    pub out_fee_token: T,
    pub out_mint: T,
    pub wsol_mint: T,
    pub token_program: T,
    pub system_program: T,
}

pub type StakeWrappedSolPrefixKeysOwned = StakeWrappedSolPrefixAccs<[u8; 32]>;
pub type StakeWrappedSolPrefixKeys<'a> = StakeWrappedSolPrefixAccs<&'a [u8; 32]>;
pub type StakeWrappedSolPrefixAccsFlag = StakeWrappedSolPrefixAccs<bool>;

pub const STAKE_WRAPPED_SOL_PREFIX_IS_WRITER: StakeWrappedSolPrefixAccsFlag =
    StakeWrappedSolPrefixAccs([false; STAKE_WRAPPED_SOL_PREFIX_ACCS_LEN])
        .const_with_out_token(true)
        .const_with_inp_wsol(true)
        .const_with_sol_bridge_out(true)
        .const_with_wsol_bridge_in(true)
        .const_with_out_mint(true)
        .const_with_out_fee_token(true);

pub const STAKE_WRAPPED_SOL_PREFIX_IS_SIGNER: StakeWrappedSolPrefixAccsFlag =
    StakeWrappedSolPrefixAccs([false; STAKE_WRAPPED_SOL_PREFIX_ACCS_LEN]).const_with_user(true);

impl<T> StakeWrappedSolPrefixAccs<T> {
    pub const fn new(arr: [T; STAKE_WRAPPED_SOL_PREFIX_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

impl StakeWrappedSolPrefixKeysOwned {
    #[inline]
    pub fn as_borrowed(&self) -> StakeWrappedSolPrefixKeys<'_> {
        StakeWrappedSolPrefixKeys::new(self.0.each_ref())
    }

    #[inline]
    pub fn with_consts(self) -> Self {
        self.as_borrowed().with_consts().into_owned()
    }
}

impl StakeWrappedSolPrefixKeys<'_> {
    #[inline]
    pub fn into_owned(self) -> StakeWrappedSolPrefixKeysOwned {
        StakeWrappedSolPrefixKeysOwned::new(self.0.map(|pk| *pk))
    }

    #[inline]
    pub const fn with_consts(self) -> Self {
        self.const_with_system_program(&SYSTEM_PROGRAM)
            .const_with_wsol_bridge_in(&WSOL_BRIDGE_IN)
            .const_with_sol_bridge_out(&SOL_BRIDGE_OUT)
    }
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StakeWrappedSolIxData([u8; 9]);

impl StakeWrappedSolIxData {
    #[inline]
    pub fn new(amt: u64) -> Self {
        let mut buf = [0u8; 9];

        buf[0] = INSTRUCTION_IDX_STAKE_WRAPPED_SOL;
        buf[1..9].copy_from_slice(&amt.to_le_bytes());

        Self(buf)
    }

    #[inline]
    pub const fn to_buf(&self) -> [u8; 9] {
        self.0
    }
}
