use generic_array_struct::generic_array_struct;

use super::INSTRUCTION_IDX_WITHDRAW_WRAPPED_SOL;

#[generic_array_struct(builder pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct WithdrawWrappedSolPrefixAccs<T> {
    pub user: T,
    pub inp_token: T,
    pub out_wsol: T,
    pub wsol_fee_token: T,
    pub inp_mint: T,
    pub wsol_mint: T,
    pub token_program: T,
}

pub type WithdrawWrappedSolPrefixKeysOwned = WithdrawWrappedSolPrefixAccs<[u8; 32]>;
pub type WithdrawWrappedSolPrefixKeys<'a> = WithdrawWrappedSolPrefixAccs<&'a [u8; 32]>;
pub type WithdrawWrappedSolPrefixAccsFlag = WithdrawWrappedSolPrefixAccs<bool>;

pub const WITHDRAW_WRAPPED_SOL_PREFIX_IS_WRITER: WithdrawWrappedSolPrefixAccsFlag =
    WithdrawWrappedSolPrefixAccs([false; WITHDRAW_WRAPPED_SOL_PREFIX_ACCS_LEN])
        .const_with_inp_token(true)
        .const_with_inp_mint(true)
        .const_with_out_wsol(true)
        .const_with_wsol_fee_token(true);

pub const WITHDRAW_WRAPPED_SOL_PREFIX_IS_SIGNER: WithdrawWrappedSolPrefixAccsFlag =
    WithdrawWrappedSolPrefixAccs([false; WITHDRAW_WRAPPED_SOL_PREFIX_ACCS_LEN])
        .const_with_user(true);

impl<T> WithdrawWrappedSolPrefixAccs<T> {
    pub const fn new(arr: [T; WITHDRAW_WRAPPED_SOL_PREFIX_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

impl WithdrawWrappedSolPrefixKeysOwned {
    pub fn as_borrowed(&self) -> WithdrawWrappedSolPrefixKeys<'_> {
        WithdrawWrappedSolPrefixKeys::new(self.0.each_ref())
    }
}

impl WithdrawWrappedSolPrefixKeys<'_> {
    pub fn into_owned(self) -> WithdrawWrappedSolPrefixKeysOwned {
        WithdrawWrappedSolPrefixKeysOwned::new(self.0.map(|pk| *pk))
    }
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WithdrawWrappedSolIxData([u8; 9]);

impl WithdrawWrappedSolIxData {
    pub fn new(amt: u64) -> Self {
        let mut buf = [0u8; 9];

        buf[0] = INSTRUCTION_IDX_WITHDRAW_WRAPPED_SOL;
        buf[1..9].copy_from_slice(&amt.to_le_bytes());

        Self(buf)
    }

    pub const fn to_buf(&self) -> [u8; 9] {
        self.0
    }
}
