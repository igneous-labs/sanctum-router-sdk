import { describe, expect, it } from "vitest";
import {
  depositSolFixturesTest,
  depositStakeFixturesTest,
  localRpc,
  parseRouterErr,
  PICOSOL_MINT,
  prefundSwapViaStakeFixturesTest,
  prefundWithdrawStakeFixturesTest,
  routerForSwaps,
  withdrawSolFixturesTest,
} from "../utils";
import { quoteWithdrawSol } from "@sanctumso/sanctum-router";

const PICOSOL_TOKEN_ACC_NAME = "signer-picosol-token";

describe("SPL Test", async () => {
  // DepositSol
  it("spl-picosol-deposit-sol", async () => {
    await depositSolFixturesTest(1000000n, {
      inp: "signer-wsol-token",
      out: PICOSOL_TOKEN_ACC_NAME,
    });
  });

  // WithdrawSol
  it("spl-picosol-withdraw-sol", async () => {
    await withdrawSolFixturesTest(1000000n, {
      inp: PICOSOL_TOKEN_ACC_NAME,
      out: "signer-wsol-token",
    });
  });

  it("spl-picosol-withdraw-sol-fails-withdrawal-too-large", async () => {
    const rpc = localRpc();
    const router = await routerForSwaps(rpc, [
      { swap: "withdrawSol", inp: PICOSOL_MINT },
    ]);
    try {
      quoteWithdrawSol(router, {
        // a very large amount
        amt: 1_000_000_000_000_000_000n,
        inp: PICOSOL_MINT,
      });
      expect.fail("should have thrown");
    } catch (e) {
      expect(e).toSatisfy((e) => {
        const [code] = parseRouterErr(e);
        return code === "PoolErr";
      });
    }
  });

  // DepositStake
  it("spl-picosol-deposit-stake", async () => {
    await depositStakeFixturesTest({
      inp: "picosol-deposit-stake",
      out: PICOSOL_TOKEN_ACC_NAME,
    });
  });

  // PrefundWithdrawStake
  it("spl-picosol-prefund-withdraw-stake", async () => {
    await prefundWithdrawStakeFixturesTest(
      1_000_000_000n,
      PICOSOL_TOKEN_ACC_NAME
    );
  });

  // PrefundSwapViaStake

  it("spl-picosol-prefund-swap-via-stake-into-reserve", async () => {
    await prefundSwapViaStakeFixturesTest(1_000_000_000n, {
      inp: PICOSOL_TOKEN_ACC_NAME,
      out: "signer-wsol-token",
    });
  });

  it("spl-picosol-prefund-swap-via-stake-into-reserve-use-bridge-vote", async () => {
    await prefundSwapViaStakeFixturesTest(
      1_000_000_000n,
      {
        inp: PICOSOL_TOKEN_ACC_NAME,
        out: "signer-wsol-token",
      },
      { useBridgeVote: true }
    );
  });

  it("spl-picosol-prefund-swap-via-stake-into-marinade", async () => {
    await prefundSwapViaStakeFixturesTest(1_000_000_000n, {
      inp: PICOSOL_TOKEN_ACC_NAME,
      out: "signer-msol-token",
    });
  });

  it("spl-picosol-prefund-swap-via-stake-into-marinade-use-bridge-vote", async () => {
    await prefundSwapViaStakeFixturesTest(
      1_000_000_000n,
      {
        inp: PICOSOL_TOKEN_ACC_NAME,
        out: "signer-msol-token",
      },
      { useBridgeVote: true }
    );
  });

  it("spl-picosol-prefund-swap-via-stake-into-spl-bsol", async () => {
    await prefundSwapViaStakeFixturesTest(1_000_000_000n, {
      inp: PICOSOL_TOKEN_ACC_NAME,
      out: "signer-bsol-token",
    });
  });

  it("spl-picosol-prefund-swap-via-stake-into-spl-bsol-use-bridge-vote", async () => {
    await prefundSwapViaStakeFixturesTest(
      1_000_000_000n,
      {
        inp: PICOSOL_TOKEN_ACC_NAME,
        out: "signer-bsol-token",
      },
      { useBridgeVote: true }
    );
  });
});
