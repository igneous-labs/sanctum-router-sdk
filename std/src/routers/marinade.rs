use sanctum_router_core::{
    sanctum_marinade_liquid_staking_core::{
        self, duplication_flag_seeds, State, ValidatorRecord, MARINADE_STAKING_PROGRAM,
    },
    MarinadeDepositSolQuoter, MarinadeDepositSolSufAccs, MarinadeDepositStakeQuoter,
    MarinadeDepositStakeSufAccs,
};

use crate::{DepositSol, DepositStake, DepositStakeAddrs};

#[derive(Debug, Clone, PartialEq)]
pub struct MarinadeRouterSol {
    pub state: State,
    pub msol_leg_balance: u64,
}

macro_rules! impl_sol_router {
    () => {
        #[inline]
        pub const fn marinade_deposit_sol_quoter(&self) -> MarinadeDepositSolQuoter<'_> {
            MarinadeDepositSolQuoter {
                state: &self.state,
                msol_leg_balance: self.msol_leg_balance,
            }
        }

        #[inline]
        pub const fn marinade_deposit_sol_suf_accs(&self) -> MarinadeDepositSolSufAccs<'_> {
            MarinadeDepositSolSufAccs::from_state(&self.state)
        }
    };
}

impl MarinadeRouterSol {
    impl_sol_router!();
}

macro_rules! impl_deposit_sol {
    () => {
        #[inline]
        fn deposit_sol_quoter(&self) -> Self::Quoter<'_> {
            self.marinade_deposit_sol_quoter()
        }

        #[inline]
        fn deposit_sol_suf_accs(&self) -> Self::SufAccs<'_> {
            self.marinade_deposit_sol_suf_accs()
        }
    };
}

impl DepositSol for MarinadeRouterSol {
    type Quoter<'a> = MarinadeDepositSolQuoter<'a>;
    type SufAccs<'a> = MarinadeDepositSolSufAccs<'a>;

    impl_deposit_sol!();
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarinadeRouter<V, F> {
    pub state: State,
    pub msol_leg_balance: u64,
    pub validator_records: V,
    pub find_pda: F,
}

impl<V, F> MarinadeRouter<V, F> {
    impl_sol_router!();
}

impl<V, F> DepositSol for MarinadeRouter<V, F> {
    type Quoter<'a>
        = MarinadeDepositSolQuoter<'a>
    where
        V: 'a,
        F: 'a;
    type SufAccs<'a>
        = MarinadeDepositSolSufAccs<'a>
    where
        V: 'a,
        F: 'a;

    impl_deposit_sol!();
}

impl<V: AsRef<[ValidatorRecord]>, F> MarinadeRouter<V, F> {
    #[inline]
    pub fn marinade_deposit_stake_quoter(&self) -> MarinadeDepositStakeQuoter<'_> {
        MarinadeDepositStakeQuoter {
            state: &self.state,
            msol_leg_balance: self.msol_leg_balance,
            validator_records: self.validator_records.as_ref(),
        }
    }
}

impl<V, F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>> MarinadeRouter<V, F> {
    #[inline]
    pub fn marinade_deposit_stake_suf_accs(
        &self,
        vote: &[u8; 32],
    ) -> Option<MarinadeDepositStakeSufAccs<'_>> {
        let (s1, s2, s3) =
            duplication_flag_seeds(&sanctum_marinade_liquid_staking_core::STATE_PUBKEY, vote);
        let find_pda = &self.find_pda;
        let (duplication_flag, _) = find_pda(
            &[s1.as_slice(), s2.as_slice(), s3.as_slice()],
            &MARINADE_STAKING_PROGRAM,
        )?;
        Some(MarinadeDepositStakeSufAccs {
            state: &self.state,
            duplication_flag,
        })
    }
}

impl<V: AsRef<[ValidatorRecord]>, F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>> DepositStake
    for MarinadeRouter<V, F>
{
    type Quoter<'a>
        = MarinadeDepositStakeQuoter<'a>
    where
        V: 'a,
        F: 'a;
    type SufAccs<'a>
        = MarinadeDepositStakeSufAccs<'a>
    where
        V: 'a,
        F: 'a;

    #[inline]
    fn deposit_stake_quoter(&self) -> Self::Quoter<'_> {
        self.marinade_deposit_stake_quoter()
    }

    #[inline]
    fn deposit_stake_suf_accs(
        &self,
        DepositStakeAddrs {
            vote: vote_addr, ..
        }: &DepositStakeAddrs,
    ) -> Option<Self::SufAccs<'_>> {
        self.marinade_deposit_stake_suf_accs(vote_addr)
    }
}
