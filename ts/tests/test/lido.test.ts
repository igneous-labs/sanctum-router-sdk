import { describe, it } from "vitest";
import {
  NATIVE_MINT,
  prefundSwapViaStakeFixturesTest,
  prefundWithdrawStakeFixturesTest,
  STSOL_MINT,
} from "../utils";

describe("Lido Test", async () => {
  it("lido-prefund-withraw-stake", async () => {
    await prefundWithdrawStakeFixturesTest(
      1_000_000_000n,
      STSOL_MINT,
      "signer-stsol-token"
    );
  });

  it("lido-prefund-swap-via-stake-into-reserve", async () => {
    await prefundSwapViaStakeFixturesTest(
      1_000_000_000n,
      {
        inp: STSOL_MINT,
        out: NATIVE_MINT,
      },
      {
        inp: "signer-stsol-token",
        out: "reserve-signer-wsol-token",
      }
    );
  });
});
