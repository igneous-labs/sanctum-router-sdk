import { describe, it } from "vitest";
import { depositSolFixturesTest, depositStakeFixturesTest } from "../utils";

const MSOL_TOKEN_ACC_NAME = "signer-msol-token";

describe("Marinade Test", async () => {
  // DepositSol
  it("marinade-deposit-sol", async () => {
    await depositSolFixturesTest(1000000n, {
      inp: "marinade-signer-wsol-token",
      out: MSOL_TOKEN_ACC_NAME,
    });
  });

  // DepositStake
  it("marinade-deposit-stake", async () => {
    await depositStakeFixturesTest({
      inp: "marinade-deposit-stake",
      out: MSOL_TOKEN_ACC_NAME,
    });
  });
});
