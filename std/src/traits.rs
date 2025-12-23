use sanctum_router_core::{
    DepositSolQuoter, DepositSolSufAccs, DepositStakeQuoter, DepositStakeSufAccs,
    WithdrawSolQuoter, WithdrawSolSufAccs, WithdrawStakeQuoter, WithdrawStakeSufAccs,
};

pub trait DepositSol {
    type Quoter<'a>: DepositSolQuoter
    where
        Self: 'a;
    type SufAccs<'a>: DepositSolSufAccs
    where
        Self: 'a;

    fn deposit_sol_quoter(&self) -> Self::Quoter<'_>;

    fn deposit_sol_suf_accs(&self) -> Self::SufAccs<'_>;
}

pub trait WithdrawSol {
    type Quoter<'a>: WithdrawSolQuoter
    where
        Self: 'a;
    type SufAccs<'a>: WithdrawSolSufAccs
    where
        Self: 'a;

    fn withdraw_sol_quoter(&self) -> Self::Quoter<'_>;

    fn withdraw_sol_suf_accs(&self) -> Self::SufAccs<'_>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DepositStakeAddrs {
    /// address of stake account to be deposited
    pub stake: [u8; 32],

    /// vote account address that `stake` is delegated to
    pub vote: [u8; 32],
}

pub trait DepositStake {
    type Quoter<'a>: DepositStakeQuoter
    where
        Self: 'a;
    type SufAccs<'a>: DepositStakeSufAccs
    where
        Self: 'a;

    fn deposit_stake_quoter(&self) -> Self::Quoter<'_>;

    /// Returns `None` if pool unable to service deposit of this validator vote acc
    fn deposit_stake_suf_accs(&self, params: &DepositStakeAddrs) -> Option<Self::SufAccs<'_>>;
}

pub trait WithdrawStake {
    type Quoter<'a>: WithdrawStakeQuoter
    where
        Self: 'a;
    type SufAccs<'a>: WithdrawStakeSufAccs
    where
        Self: 'a;

    fn withdraw_stake_quoter(&self) -> Self::Quoter<'_>;

    /// Returns `None` if pool unable to service withdrawal of this validator vote acc
    fn withdraw_stake_suf_accs(&self, vote: &[u8; 32]) -> Option<Self::SufAccs<'_>>;
}
