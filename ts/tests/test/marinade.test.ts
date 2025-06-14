import { describe, it } from "vitest";
import {
  depositSolFixturesTest,
  depositStakeFixturesTest,
  MSOL_MINT,
} from "../utils";

const MSOL_TOKEN_ACC_NAME = "signer-msol-token";

describe("Marinade Test", async () => {
  // DepositSol
  it("marinade-deposit-sol", async () => {
    await depositSolFixturesTest(1000000n, MSOL_MINT, {
      inp: "marinade-signer-wsol-token",
      out: MSOL_TOKEN_ACC_NAME,
    });
  });

  // DepositStake
  it("marinade-deposit-stake", async () => {
    await depositStakeFixturesTest(MSOL_MINT, {
      inp: "marinade-deposit-stake",
      out: MSOL_TOKEN_ACC_NAME,
    });
  });
});
