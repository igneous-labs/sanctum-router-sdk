import { describe, it } from "vitest";
import {
  NATIVE_MINT,
  prefundSwapViaStakeFixturesTest,
  prefundWithdrawStakeFixturesTest,
  STSOL_MINT,
} from "../utils";

const STSOL_TOKEN_ACC_NAME = "signer-stsol-token";

describe("Lido Test", async () => {
  // PrefundWithdrawStake
  it("lido-prefund-withraw-stake", async () => {
    await prefundWithdrawStakeFixturesTest(
      1_000_000_000n,
      STSOL_MINT,
      STSOL_TOKEN_ACC_NAME
    );
  });

  // PrefundSwapViaStake

  it("lido-prefund-swap-via-stake-into-reserve", async () => {
    await prefundSwapViaStakeFixturesTest(
      1_000_000_000n,
      {
        inp: STSOL_MINT,
        out: NATIVE_MINT,
      },
      {
        inp: STSOL_TOKEN_ACC_NAME,
        out: "reserve-signer-wsol-token",
      }
    );
  });
});
