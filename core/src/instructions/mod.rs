mod deposit_stake;
mod prefund_swap_via_stake;
mod prefund_withdraw_stake;
mod stake_wrapped_sol;
mod withdraw_wrapped_sol;

pub use deposit_stake::*;
pub use prefund_swap_via_stake::*;
pub use prefund_withdraw_stake::*;
pub use stake_wrapped_sol::*;
pub use withdraw_wrapped_sol::*;

use crate::internal_utils::seqconsts;

seqconsts!(
    ty = u8;
    count = INSTRUCTION_COUNT;

    INSTRUCTION_IDX_STAKE_WRAPPED_SOL,
    INSTRUCTION_IDX_SWAP_VIA_STAKE,
    INSTRUCTION_IDX_CREATE_FEE_TOKEN_ACCOUNT,
    INSTRUCTION_IDX_CLOSE_FEE_TOKEN_ACCOUNT,
    INSTRUCTION_IDX_WITHDRAW_FEES,
    INSTRUCTION_IDX_DEPOSIT_STAKE,
    INSTRUCTION_IDX_PREFUND_WITHDRAW_STAKE,
    INSTRUCTION_IDX_PREFUND_SWAP_VIA_STAKE,
    INSTRUCTION_IDX_WITHDRAW_WRAPPED_SOL
);
