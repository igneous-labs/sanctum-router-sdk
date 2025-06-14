import { describe, it } from "vitest";
import {
  depositSolFixturesTest,
  depositStakeFixturesTest,
  MSOL_MINT,
  NATIVE_MINT,
  PICOSOL_MINT,
  prefundSwapViaStakeFixturesTest,
  prefundWithdrawStakeFixturesTest,
  withdrawSolFixturesTest,
} from "../utils";

const PICOSOL_TOKEN_ACC_NAME = "signer-picosol-token";

describe("SPL Test", async () => {
  // DepositSol
  it("spl-picosol-deposit-sol", async () => {
    await depositSolFixturesTest(1000000n, PICOSOL_MINT, {
      inp: "spl-signer-wsol-token",
      out: PICOSOL_TOKEN_ACC_NAME,
    });
  });

  // WithdrawSol
  it("spl-picosol-withdraw-sol", async () => {
    await withdrawSolFixturesTest(1000000n, PICOSOL_MINT, {
      inp: PICOSOL_TOKEN_ACC_NAME,
      out: "spl-signer-wsol-token",
    });
  });

  // DepositStake
  it("spl-picosol-deposit-stake", async () => {
    await depositStakeFixturesTest(PICOSOL_MINT, {
      inp: "picosol-deposit-stake",
      out: PICOSOL_TOKEN_ACC_NAME,
    });
  });

  // PrefundWithdrawStake
  it("spl-picosol-prefund-withdraw-stake", async () => {
    await prefundWithdrawStakeFixturesTest(
      1_000_000_000n,
      PICOSOL_MINT,
      PICOSOL_TOKEN_ACC_NAME
    );
  });

  // PrefundSwapViaStake

  it("spl-picosol-prefund-swap-via-stake-into-reserve", async () => {
    await prefundSwapViaStakeFixturesTest(
      1_000_000_000n,
      {
        inp: PICOSOL_MINT,
        out: NATIVE_MINT,
      },
      {
        inp: PICOSOL_TOKEN_ACC_NAME,
        out: "reserve-signer-wsol-token",
      }
    );
  });

  it("spl-picosol-prefund-swap-via-stake-into-marinade", async () => {
    await prefundSwapViaStakeFixturesTest(
      1_000_000_000n,
      {
        inp: PICOSOL_MINT,
        out: MSOL_MINT,
      },
      {
        inp: PICOSOL_TOKEN_ACC_NAME,
        out: "signer-msol-token",
      }
    );
  });
});
