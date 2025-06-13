use solido_legacy_core::validator_stake_seeds;

use crate::pda::find_pda;

pub fn find_lido_validator_stake_account_pda_internal(
    vote_account: &[u8; 32],
    seed: u64,
) -> Option<([u8; 32], u8)> {
    let (s1, s2, s3, s4) = validator_stake_seeds(vote_account, seed);
    find_pda(
        &[s1.as_slice(), s2.as_slice(), s3.as_slice(), s4.as_slice()],
        &solido_legacy_core::PROGRAM_ID,
    )
}
