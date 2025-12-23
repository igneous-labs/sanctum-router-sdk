use sanctum_router_core::{
    sanctum_spl_stake_pool_core::{validator_stake_seeds, StakePool, ValidatorStakeInfo},
    SplDepositSolQuoter, SplDepositStakeQuoter, SplDepositStakeSufAccs, SplSolSufAccs,
    SplWithdrawSolQuoter, SplWithdrawStakeQuoter, SplWithdrawStakeSufAccs,
};

use crate::{DepositSol, DepositStake, DepositStakeAddrs, WithdrawSol, WithdrawStake};

#[derive(Debug, Clone, PartialEq)]
pub struct SplRouterDepositSol {
    pub stake_pool_program: [u8; 32],
    pub stake_pool_addr: [u8; 32],
    pub withdraw_authority_program_address: [u8; 32],
    pub stake_pool: StakePool,
    pub curr_epoch: u64,
}

macro_rules! impl_deposit_sol_quoter {
    () => {
        #[inline]
        pub const fn spl_deposit_sol_quoter(&self) -> SplDepositSolQuoter<'_> {
            SplDepositSolQuoter {
                stake_pool: &self.stake_pool,
                curr_epoch: self.curr_epoch,
            }
        }
    };
}

macro_rules! impl_sol_suf_accs {
    () => {
        #[inline]
        pub const fn spl_sol_suf_accs(&self) -> SplSolSufAccs<'_> {
            SplSolSufAccs {
                stake_pool: &self.stake_pool,
                stake_pool_program: &self.stake_pool_program,
                stake_pool_addr: &self.stake_pool_addr,
                withdraw_authority_program_address: &self.withdraw_authority_program_address,
            }
        }
    };
}

impl SplRouterDepositSol {
    impl_deposit_sol_quoter!();
    impl_sol_suf_accs!();
}

macro_rules! impl_deposit_sol {
    () => {
        #[inline]
        fn deposit_sol_quoter(&self) -> Self::Quoter<'_> {
            self.spl_deposit_sol_quoter()
        }

        #[inline]
        fn deposit_sol_suf_accs(&self) -> Self::SufAccs<'_> {
            self.spl_sol_suf_accs()
        }
    };
}

impl DepositSol for SplRouterDepositSol {
    type Quoter<'a> = SplDepositSolQuoter<'a>;
    type SufAccs<'a> = SplSolSufAccs<'a>;

    impl_deposit_sol!();
}

#[derive(Debug, Clone, PartialEq)]
pub struct SplRouterSol {
    pub stake_pool_program: [u8; 32],
    pub stake_pool_addr: [u8; 32],
    pub withdraw_authority_program_address: [u8; 32],
    pub stake_pool: StakePool,
    pub curr_epoch: u64,

    pub reserve_stake_lamports: u64,
}

macro_rules! impl_withdraw_sol_quoter {
    () => {
        #[inline]
        pub const fn spl_withdraw_sol_quoter(&self) -> SplWithdrawSolQuoter<'_> {
            SplWithdrawSolQuoter {
                stake_pool: &self.stake_pool,
                curr_epoch: self.curr_epoch,
                reserve_stake_lamports: self.reserve_stake_lamports,
            }
        }
    };
}

macro_rules! impl_sol_router {
    () => {
        impl_deposit_sol_quoter!();
        impl_withdraw_sol_quoter!();
        impl_sol_suf_accs!();
    };
}

impl SplRouterSol {
    impl_sol_router!();
}

impl DepositSol for SplRouterSol {
    type Quoter<'a> = SplDepositSolQuoter<'a>;
    type SufAccs<'a> = SplSolSufAccs<'a>;

    impl_deposit_sol!();
}

macro_rules! impl_withdraw_sol {
    () => {
        #[inline]
        fn withdraw_sol_quoter(&self) -> Self::Quoter<'_> {
            self.spl_withdraw_sol_quoter()
        }

        #[inline]
        fn withdraw_sol_suf_accs(&self) -> Self::SufAccs<'_> {
            self.spl_sol_suf_accs()
        }
    };
}

impl WithdrawSol for SplRouterSol {
    type Quoter<'a> = SplWithdrawSolQuoter<'a>;
    type SufAccs<'a> = SplSolSufAccs<'a>;

    impl_withdraw_sol!();
}

#[derive(Debug, Clone, PartialEq)]
pub struct SplRouterStake<V, F> {
    pub stake_pool_program: [u8; 32],
    pub stake_pool_addr: [u8; 32],
    pub withdraw_authority_program_address: [u8; 32],
    pub stake_pool: StakePool,
    pub curr_epoch: u64,

    pub default_stake_deposit_authority: [u8; 32],
    pub validator_list: V,
    pub find_pda: F,
}

macro_rules! impl_stake_quoter {
    () => {
        #[inline]
        pub fn spl_deposit_stake_quoter(&self) -> SplDepositStakeQuoter<'_> {
            SplDepositStakeQuoter {
                stake_pool: &self.stake_pool,
                curr_epoch: self.curr_epoch,
                validator_list: self.validator_list.as_ref(),
                default_stake_deposit_authority: &self.default_stake_deposit_authority,
            }
        }

        #[inline]
        pub fn spl_withdraw_stake_quoter(&self) -> SplWithdrawStakeQuoter<'_> {
            SplWithdrawStakeQuoter {
                stake_pool: &self.stake_pool,
                curr_epoch: self.curr_epoch,
                validator_list: self.validator_list.as_ref(),
            }
        }
    };
}

impl<V: AsRef<[ValidatorStakeInfo]>, F> SplRouterStake<V, F> {
    impl_stake_quoter!();
}

macro_rules! impl_stake_accs {
    () => {
        #[inline]
        pub fn find_vsa_addr(&self, vote: &[u8; 32]) -> Option<[u8; 32]> {
            let vsi = self
                .validator_list
                .as_ref()
                .iter()
                .find(|v| v.vote_account_address() == vote)?;
            let (s1, s2, s3) =
                validator_stake_seeds(vote, &self.stake_pool_addr, vsi.validator_seed_suffix());
            let find_pda = &self.find_pda;
            let (validator_stake, _) = find_pda(
                &[s1.as_slice(), s2.as_slice(), s3.as_slice()],
                &self.stake_pool_program,
            )?;
            Some(validator_stake)
        }

        #[inline]
        pub fn spl_deposit_stake_suf_accs(
            &self,
            vote: &[u8; 32],
        ) -> Option<SplDepositStakeSufAccs<'_>> {
            Some(SplDepositStakeSufAccs {
                stake_pool_addr: &self.stake_pool_addr,
                stake_pool_program: &self.stake_pool_program,
                stake_pool: &self.stake_pool,
                stake_deposit_authority: &self.stake_pool.stake_deposit_authority,
                stake_withdraw_authority: &self.withdraw_authority_program_address,
                validator_stake: self.find_vsa_addr(vote)?,
            })
        }

        #[inline]
        pub fn spl_withdraw_stake_suf_accs(
            &self,
            vote: &[u8; 32],
        ) -> Option<SplWithdrawStakeSufAccs<'_>> {
            Some(SplWithdrawStakeSufAccs {
                stake_pool_addr: &self.stake_pool_addr,
                stake_pool_program: &self.stake_pool_program,
                stake_pool: &self.stake_pool,
                stake_withdraw_authority: &self.withdraw_authority_program_address,
                validator_stake: self.find_vsa_addr(vote)?,
            })
        }
    };
}

impl<V: AsRef<[ValidatorStakeInfo]>, F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>>
    SplRouterStake<V, F>
{
    impl_stake_accs!();
}

macro_rules! impl_deposit_stake {
    () => {
        type Quoter<'a>
            = SplDepositStakeQuoter<'a>
        where
            F: 'a,
            V: 'a;

        type SufAccs<'a>
            = SplDepositStakeSufAccs<'a>
        where
            F: 'a,
            V: 'a;

        #[inline]
        fn deposit_stake_quoter(&self) -> Self::Quoter<'_> {
            self.spl_deposit_stake_quoter()
        }

        #[inline]
        fn deposit_stake_suf_accs(
            &self,
            DepositStakeAddrs { vote, .. }: &DepositStakeAddrs,
        ) -> Option<Self::SufAccs<'_>> {
            self.spl_deposit_stake_suf_accs(vote)
        }
    };
}

impl<V: AsRef<[ValidatorStakeInfo]>, F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>>
    DepositStake for SplRouterStake<V, F>
{
    impl_deposit_stake!();
}

macro_rules! impl_withdraw_stake {
    () => {
        type Quoter<'a>
            = SplWithdrawStakeQuoter<'a>
        where
            F: 'a,
            V: 'a;

        type SufAccs<'a>
            = SplWithdrawStakeSufAccs<'a>
        where
            F: 'a,
            V: 'a;

        #[inline]
        fn withdraw_stake_quoter(&self) -> Self::Quoter<'_> {
            self.spl_withdraw_stake_quoter()
        }

        #[inline]
        fn withdraw_stake_suf_accs(&self, vote: &[u8; 32]) -> Option<Self::SufAccs<'_>> {
            self.spl_withdraw_stake_suf_accs(vote)
        }
    };
}

impl<V: AsRef<[ValidatorStakeInfo]>, F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>>
    WithdrawStake for SplRouterStake<V, F>
{
    impl_withdraw_stake!();
}

impl<V, F> SplRouterStake<V, F> {
    impl_deposit_sol_quoter!();
    impl_sol_suf_accs!();
}

impl<V, F> DepositSol for SplRouterStake<V, F> {
    type Quoter<'a>
        = SplDepositSolQuoter<'a>
    where
        F: 'a,
        V: 'a;
    type SufAccs<'a>
        = SplSolSufAccs<'a>
    where
        F: 'a,
        V: 'a;

    impl_deposit_sol!();
}

#[derive(Debug, Clone, PartialEq)]
pub struct SplRouter<V, F> {
    pub stake_pool_program: [u8; 32],
    pub stake_pool_addr: [u8; 32],
    pub withdraw_authority_program_address: [u8; 32],
    pub stake_pool: StakePool,
    pub curr_epoch: u64,

    pub reserve_stake_lamports: u64,

    pub default_stake_deposit_authority: [u8; 32],
    pub validator_list: V,
    pub find_pda: F,
}

impl<V, F> SplRouter<V, F> {
    impl_sol_router!();
}

impl<V: AsRef<[ValidatorStakeInfo]>, F> SplRouter<V, F> {
    impl_stake_quoter!();
}

impl<V: AsRef<[ValidatorStakeInfo]>, F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>>
    SplRouter<V, F>
{
    impl_stake_accs!();
}

impl<V, F> DepositSol for SplRouter<V, F> {
    type Quoter<'a>
        = SplDepositSolQuoter<'a>
    where
        F: 'a,
        V: 'a;
    type SufAccs<'a>
        = SplSolSufAccs<'a>
    where
        F: 'a,
        V: 'a;

    impl_deposit_sol!();
}

impl<V, F> WithdrawSol for SplRouter<V, F> {
    type Quoter<'a>
        = SplWithdrawSolQuoter<'a>
    where
        F: 'a,
        V: 'a;
    type SufAccs<'a>
        = SplSolSufAccs<'a>
    where
        F: 'a,
        V: 'a;

    impl_withdraw_sol!();
}

impl<V: AsRef<[ValidatorStakeInfo]>, F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>>
    DepositStake for SplRouter<V, F>
{
    impl_deposit_stake!();
}

impl<V: AsRef<[ValidatorStakeInfo]>, F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>>
    WithdrawStake for SplRouter<V, F>
{
    impl_withdraw_stake!();
}
