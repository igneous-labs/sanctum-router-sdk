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
pub struct StakeAccountLamports {
    /// Actively staked lamports of this stake account
    pub staked: u64,

    /// Unstaked lamports of this stake account. Can be rent-exemption, MEV tips etc
    pub unstaked: u64,
}

impl StakeAccountLamports {
    pub const fn total(&self) -> u64 {
        self.staked + self.unstaked
    }
}

// not deriving serde and wasm for this because of [u8; 32]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActiveStakeParams {
    pub vote: [u8; 32],
    pub lamports: StakeAccountLamports,
}
