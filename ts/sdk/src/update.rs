use sanctum_router_core::NATIVE_MINT;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::JsError;

use crate::interface::{AccountMap, B58PK};

pub trait Update {
    /// Returns empty iterator for [`PoolUpdateType`]s not supported by the pool
    fn accounts_to_update(&self, ty: PoolUpdateType) -> impl Iterator<Item = [u8; 32]>;

    /// Returns error if
    /// - [`PoolUpdateType`] `ty` not supported by the pool
    /// - update procedure failed (e.g. account requested in [`accounts_to_update`] not present)
    fn update(&mut self, ty: PoolUpdateType, accounts: &AccountMap) -> Result<(), JsError>;
}

/// - `inp` input mint
/// - `out` output mint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub enum SwapMints {
    DepositSol { out: B58PK },
    DepositStake { out: B58PK },
    PrefundSwapViaStake { inp: B58PK, out: B58PK },
    WithdrawSol { inp: B58PK },
    PrefundWithdrawStake { inp: B58PK },
}

pub(crate) type IntoPoolUpdateIter =
    core::iter::Flatten<core::array::IntoIter<Option<PoolUpdate>, 3>>;

impl SwapMints {
    #[inline]
    pub(crate) fn into_pool_updates(self) -> IntoPoolUpdateIter {
        match self {
            SwapMints::DepositSol { out } => [
                Some(PoolUpdate {
                    mint: out.0,
                    ty: PoolUpdateType::DepositSol,
                }),
                None,
                None,
            ],

            SwapMints::DepositStake { out } => [
                Some(PoolUpdate {
                    mint: out.0,
                    ty: PoolUpdateType::DepositStake,
                }),
                None,
                None,
            ],

            SwapMints::PrefundSwapViaStake { inp, out } => [
                PoolUpdate {
                    mint: inp.0,
                    ty: PoolUpdateType::WithdrawStake,
                },
                PoolUpdate {
                    mint: out.0,
                    ty: PoolUpdateType::DepositStake,
                },
                // reserve pool for prefund
                PoolUpdate {
                    mint: NATIVE_MINT,
                    ty: PoolUpdateType::DepositStake,
                },
            ]
            .map(Some),

            SwapMints::WithdrawSol { inp } => [
                Some(PoolUpdate {
                    mint: inp.0,
                    ty: PoolUpdateType::WithdrawSol,
                }),
                None,
                None,
            ],

            SwapMints::PrefundWithdrawStake { inp } => [
                Some(PoolUpdate {
                    mint: inp.0,
                    ty: PoolUpdateType::WithdrawStake,
                }),
                // reserve pool for prefund
                Some(PoolUpdate {
                    mint: NATIVE_MINT,
                    ty: PoolUpdateType::DepositStake,
                }),
                None,
            ],
        }
        .into_iter()
        .flatten()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PoolUpdateType {
    DepositSol,
    DepositStake,
    WithdrawSol,
    WithdrawStake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolUpdate {
    pub mint: [u8; 32],
    pub ty: PoolUpdateType,
}
