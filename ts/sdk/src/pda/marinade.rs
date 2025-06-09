use sanctum_marinade_liquid_staking_core::{duplication_flag_seeds, MARINADE_STAKING_PROGRAM};

use crate::pda::find_pda;

/// Marinade Duplication Flag
pub fn find_marinade_duplication_flag_pda_internal(
    vote_account: &[u8; 32],
) -> Option<([u8; 32], u8)> {
    let (s1, s2, s3) = duplication_flag_seeds(
        &sanctum_marinade_liquid_staking_core::STATE_PUBKEY,
        vote_account,
    );
    find_pda(
        &[s1.as_slice(), s2.as_slice(), s3.as_slice()],
        &MARINADE_STAKING_PROGRAM,
    )
}
