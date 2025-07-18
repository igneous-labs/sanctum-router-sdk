use generic_array_struct::generic_array_struct;

use super::INSTRUCTION_IDX_DEPOSIT_STAKE;

#[generic_array_struct(builder pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DepositStakeIxAccs<T> {
    pub user: T,
    pub inp_stake: T,
    pub out_token: T,
    pub out_fee_token: T,
    pub out_mint: T,
}
pub type DepositStakeIxKeysOwned = DepositStakeIxAccs<[u8; 32]>;
pub type DepositStakeIxKeys<'a> = DepositStakeIxAccs<&'a [u8; 32]>;
pub type DepositStakeIxAccsFlag = DepositStakeIxAccs<bool>;

/// To support Marinade's DepositStake, `user` needs to be mutable
pub const DEPOSIT_STAKE_IX_IS_WRITER_NON_WSOL_OUT: DepositStakeIxAccsFlag =
    DepositStakeIxAccs([true; DEPOSIT_STAKE_IX_ACCS_LEN]);

/// If output mint is wsol, it must be set to readonly
pub const DEPOSIT_STAKE_IX_IS_WRITER_WSOL_OUT: DepositStakeIxAccsFlag =
    DEPOSIT_STAKE_IX_IS_WRITER_NON_WSOL_OUT.const_with_out_mint(false);

pub const DEPOSIT_STAKE_IX_IS_SIGNER: DepositStakeIxAccsFlag =
    DepositStakeIxAccs([false; DEPOSIT_STAKE_IX_ACCS_LEN]).const_with_user(true);

impl<T> DepositStakeIxAccs<T> {
    pub const fn new(arr: [T; DEPOSIT_STAKE_IX_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

impl<T> AsRef<[T]> for DepositStakeIxAccs<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl DepositStakeIxKeysOwned {
    pub fn as_borrowed(&self) -> DepositStakeIxKeys<'_> {
        DepositStakeIxKeys::new(self.0.each_ref())
    }
}

impl DepositStakeIxKeys<'_> {
    pub fn into_owned(self) -> DepositStakeIxKeysOwned {
        DepositStakeIxKeysOwned::new(self.0.map(|pk| *pk))
    }
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DepositStakeIxData([u8; 1]);

impl DepositStakeIxData {
    #[inline]
    pub const fn new() -> Self {
        Self([INSTRUCTION_IDX_DEPOSIT_STAKE])
    }

    #[inline]
    pub const fn to_buf(&self) -> [u8; 1] {
        self.0
    }
}
