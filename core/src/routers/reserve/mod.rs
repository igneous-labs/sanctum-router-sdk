use sanctum_reserve_core::{Fee, Pool, ProtocolFee};

mod deposit_stake;

#[derive(Debug, Clone)]
pub struct ReserveRouter<'a> {
    pub pool: &'a Pool,
    pub fee_account: &'a Fee,
    pub protocol_fee_account: &'a ProtocolFee,
    pub pool_sol_reserves: u64,
    pub stake_acc_record_addr: [u8; 32],
}
