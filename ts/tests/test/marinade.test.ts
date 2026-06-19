import { describe, expect, it } from "vitest";
import { depositSolFixturesTest, depositStakeFixturesTest } from "../utils";

const MSOL_TOKEN_ACC_NAME = "signer-msol-token";

describe("Marinade Test", async () => {
  // DepositSol
  it("marinade-deposit-sol", async () => {
    const q = await depositSolFixturesTest(1000000n, {
      inp: "signer-wsol-token",
      out: MSOL_TOKEN_ACC_NAME,
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "quote": {
          "fee": 0n,
          "inp": 1000000n,
          "out": 776062n,
        },
        "routerFee": 0n,
      }
    `);
  });

  // DepositStake
  it("marinade-deposit-stake", async () => {
    const q = await depositStakeFixturesTest({
      inp: "marinade-deposit-stake",
      out: MSOL_TOKEN_ACC_NAME,
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "quote": {
          "fee": 0n,
          "inp": {
            "staked": 1000000000n,
            "unstaked": 2282880n,
          },
          "out": 775286591n,
          "vote": "BLADE1qNA1uNjRgER6DtUFf7FU3c1TWLLdpPeEcKatZ2",
        },
        "routerFee": 776062n,
      }
    `);
  });
});
