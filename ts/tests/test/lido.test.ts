import { describe, it } from "vitest";
import {
  prefundSwapViaStakeFixturesTest,
  prefundWithdrawStakeFixturesTest,
} from "../utils";

const STSOL_TOKEN_ACC_NAME = "signer-stsol-token";

describe("Lido Test", async () => {
  // PrefundWithdrawStake
  it("lido-prefund-withraw-stake", async () => {
    await prefundWithdrawStakeFixturesTest(
      1_000_000_000n,
      STSOL_TOKEN_ACC_NAME
    );
  });

  // PrefundSwapViaStake

  it("lido-prefund-swap-via-stake-into-reserve", async () => {
    await prefundSwapViaStakeFixturesTest(1_000_000_000n, {
      inp: STSOL_TOKEN_ACC_NAME,
      out: "signer-wsol-token",
    });
  });

  it("lido-prefund-swap-via-stake-into-reserve-use-bridge-vote", async () => {
    await prefundSwapViaStakeFixturesTest(
      1_000_000_000n,
      {
        inp: STSOL_TOKEN_ACC_NAME,
        out: "signer-wsol-token",
      },
      {
        useBridgeVote: true,
      }
    );
  });

  it("lido-prefund-swap-via-stake-into-marinade", async () => {
    await prefundSwapViaStakeFixturesTest(1_000_000_000n, {
      inp: STSOL_TOKEN_ACC_NAME,
      out: "signer-msol-token",
    });
  });

  it("lido-prefund-swap-via-stake-into-marinade-use-bridge-vote", async () => {
    await prefundSwapViaStakeFixturesTest(
      1_000_000_000n,
      {
        inp: STSOL_TOKEN_ACC_NAME,
        out: "signer-msol-token",
      },
      {
        useBridgeVote: true,
      }
    );
  });

  it("lido-prefund-swap-via-stake-into-spl-bsol", async () => {
    await prefundSwapViaStakeFixturesTest(1_000_000_000n, {
      inp: STSOL_TOKEN_ACC_NAME,
      out: "signer-bsol-token",
    });
  });

  it("lido-prefund-swap-via-stake-into-spl-bsol-use-bridge-vote", async () => {
    await prefundSwapViaStakeFixturesTest(
      1_000_000_000n,
      {
        inp: STSOL_TOKEN_ACC_NAME,
        out: "signer-bsol-token",
      },
      { useBridgeVote: true }
    );
  });
});
