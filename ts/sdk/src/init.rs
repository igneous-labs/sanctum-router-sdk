use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use crate::interface::B58PK;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase", tag = "pool")]
pub enum InitData {
    Spl(SplInitData),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct SplInitData {
    pub stake_pool_addr: B58PK,

    /// Can be read from stake pool account owner
    pub stake_pool_program_addr: B58PK,

    /// Can be read from stake pool account data
    pub validator_list_addr: B58PK,

    /// Can be read from stake pool account data
    pub reserve_stake_addr: B58PK,
}
