use sanctum_router_core::{
    PrefundWithdrawStakeIxData, PrefundWithdrawStakePrefixAccsBuilder, PREFUNDER,
    PREFUND_WITHDRAW_STAKE_PREFIX_ACCS_LEN, PREFUND_WITHDRAW_STAKE_PREFIX_IS_SIGNER,
    PREFUND_WITHDRAW_STAKE_PREFIX_IS_WRITER, STAKE_PROGRAM, SYSTEM_PROGRAM, SYSVAR_CLOCK,
};
use wasm_bindgen::JsError;

use crate::{
    err::invalid_pda_err,
    interface::{keys_signer_writer_to_account_metas, AccountMeta, WithdrawStakeSwapParams},
    pda::{
        reserve::find_reserve_stake_account_record_pda_internal,
        router::{create_slumdog_stake_internal, find_bridge_stake_acc_internal},
    },
};

pub(crate) fn get_prefund_withdraw_stake_prefix_metas_and_data(
    swap_params: &WithdrawStakeSwapParams,
) -> Result<
    (
        [AccountMeta; PREFUND_WITHDRAW_STAKE_PREFIX_ACCS_LEN],
        PrefundWithdrawStakeIxData,
    ),
    JsError,
> {
    let (bridge_stake, _bump) =
        find_bridge_stake_acc_internal(&swap_params.signer.0, swap_params.bridge_stake_seed)
            .ok_or_else(invalid_pda_err)?;
    let slumdog_stake = create_slumdog_stake_internal(&bridge_stake);
    let (slumdog_stake_acc_record, _bump) =
        find_reserve_stake_account_record_pda_internal(&slumdog_stake)
            .ok_or_else(invalid_pda_err)?;
    let metas = keys_signer_writer_to_account_metas(
        &PrefundWithdrawStakePrefixAccsBuilder::start()
            .with_user(swap_params.signer.0)
            .with_bridge_stake(bridge_stake)
            .with_slumdog_stake(slumdog_stake)
            .with_slumdog_stake_acc_record(slumdog_stake_acc_record)
            .with_inp_mint(swap_params.inp.0)
            .with_inp_token(swap_params.signer_inp.0)
            .with_clock(SYSVAR_CLOCK)
            .with_prefunder(PREFUNDER)
            .with_stake_program(STAKE_PROGRAM)
            .with_system_program(SYSTEM_PROGRAM)
            .with_unstake_program(sanctum_reserve_core::UNSTAKE_PROGRAM)
            .with_unstake_pool(sanctum_reserve_core::POOL)
            .with_unstake_fee(sanctum_reserve_core::FEE)
            .with_unstake_pool_sol_reserves(sanctum_reserve_core::POOL_SOL_RESERVES)
            .with_unstake_protocol_fee(sanctum_reserve_core::PROTOCOL_FEE)
            .with_unstake_protocol_fee_dest(sanctum_reserve_core::PROTOCOL_FEE_VAULT)
            .build()
            .as_borrowed()
            .0,
        &PREFUND_WITHDRAW_STAKE_PREFIX_IS_SIGNER.0,
        &PREFUND_WITHDRAW_STAKE_PREFIX_IS_WRITER.0,
    );

    let data = PrefundWithdrawStakeIxData::new(swap_params.amt, swap_params.bridge_stake_seed);

    Ok((metas, data))
}
