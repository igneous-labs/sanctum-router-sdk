import { BSOL_MINT, PICOSOL_MINT } from "./token";

export const BSOL_INIT_DATA = {
  stakePoolAddr: "stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi",
  stakePoolProgramAddr: "SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy",
  validatorListAddr: "1istpXjy8BM7Vd5vPfA485frrV7SRJhgq5vs3sskWmc",
  reserveStakeAddr: "rsrxDvYUXjH1RQj2Ke36LNZEVqGztATxFkqNukERqFT",
} as const;

export const PICOSOL_INIT_DATA = {
  stakePoolAddr: "8Dv3hNYcEWEaa4qVx9BTN1Wfvtha1z8cWDUXb7KVACVe",
  stakePoolProgramAddr: "SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY",
  validatorListAddr: "46A5KjX8J6FAUTXwcE8iJkmM7igK3v8vy1MD74cZNWVE",
  reserveStakeAddr: "2ArodFTZhNqVWJT92qEGDxigAvouSo1kfgfEcC3KEWUK",
} as const;

// TODO: bsol and picosol are currently the only SPL pools being tested.
// May need to add to this list in the future if we add more.
export const SPL_INIT_HARDCODES = {
  [BSOL_MINT]: BSOL_INIT_DATA,
  [PICOSOL_MINT]: PICOSOL_INIT_DATA,
} as const;
