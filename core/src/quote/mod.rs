mod deposit_stake;
mod prefund;
mod token;
mod withdraw_stake;

pub use deposit_stake::*;
pub use prefund::*;
pub use token::*;
pub use withdraw_stake::*;

/// A quote with the sanctum router global fee charged on top.
///
/// Total fees = `fee of self.quote + self.router_fee`,
/// and is always in terms of output tokens
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)
)]
pub struct WithRouterFee<Q> {
    pub quote: Q,

    /// This is always in terms of `quote`'s output tokens
    pub router_fee: u64,
}

impl<Q> WithRouterFee<Q> {
    /// Returns `quote` unchanged with 0 router fees
    #[inline]
    pub const fn zero(quote: Q) -> Self {
        Self {
            quote,
            router_fee: 0,
        }
    }
}
