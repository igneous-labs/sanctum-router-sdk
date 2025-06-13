use const_crypto::bs58;
use sanctum_spl_stake_pool_core::STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS;

pub const SYSVAR_RENT: [u8; 32] =
    bs58::decode_pubkey("SysvarRent111111111111111111111111111111111");

pub const SYSVAR_STAKE_HISTORY: [u8; 32] =
    bs58::decode_pubkey("SysvarStakeHistory1111111111111111111111111");

pub const SYSVAR_CLOCK: [u8; 32] =
    bs58::decode_pubkey("SysvarC1ock11111111111111111111111111111111");

pub const STAKE_PROGRAM: [u8; 32] =
    bs58::decode_pubkey("Stake11111111111111111111111111111111111111");

pub const SYSVAR_STAKE_CONFIG: [u8; 32] =
    bs58::decode_pubkey("StakeConfig11111111111111111111111111111111");

pub const SYSTEM_PROGRAM: [u8; 32] = bs58::decode_pubkey("11111111111111111111111111111111");

pub const TOKEN_PROGRAM: [u8; 32] =
    bs58::decode_pubkey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub const ASSOCIATED_TOKEN_PROGRAM: [u8; 32] =
    bs58::decode_pubkey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

pub const SANCTUM_ROUTER_PROGRAM: [u8; 32] =
    bs58::decode_pubkey("stkitrT1Uoy18Dk1fTrgPw8W6MVzoCfYoAFT4MLsmhq");

pub const WSOL_BRIDGE_IN: [u8; 32] =
    bs58::decode_pubkey("7UWZDKjBT1dTvAzdjoSCYKnML3SPt9tfFkANGarEq5r3");

pub const SOL_BRIDGE_OUT: [u8; 32] =
    bs58::decode_pubkey("75jTZDE78xpBJokeB2BcimRNY5BZ7U45bWhpgUrTzWZC");

pub const WSOL_FEE_TOKEN_ACCOUNT: [u8; 32] =
    bs58::decode_pubkey("D3DxbHp7YvgdD2iH8GfsGWdFE5gp37aoYjp4jW5jNMjH");

pub const WITHDRAW_WRAPPED_SOL_GLOBAL_FEE_BPS: u64 = 1;

pub const DEPOSIT_STAKE_GLOBAL_FEE_BPS: u64 = 10;

pub const NATIVE_MINT: [u8; 32] =
    bs58::decode_pubkey("So11111111111111111111111111111111111111112");

pub const PREFUNDER: [u8; 32] = bs58::decode_pubkey("ALpzvhALRr35nH8mw9SXk2WvmwEYjfw1dvmpFG9Kosu6");

// TODO: STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS will change with:
// - dynamic rent
// - SOL minimum delegation feature
/// The flash loan amount given out by the router program to make the slumdog stake and withdrawn stake rent-exempt.
/// This amount is repaid by instant unstaking the slumdog stake
pub const PREFUND_FLASH_LOAN_LAMPORTS: u64 = 2 * STAKE_ACCOUNT_RENT_EXEMPT_LAMPORTS;

// hardcode for simplicity. Need to refactor when rent becomes variable.
pub const ZERO_DATA_ACC_RENT_EXEMPT_LAMPORTS: u64 = 890_880;
