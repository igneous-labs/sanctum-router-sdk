import { describe, expect, it } from "vitest";
import {
  BSOL_MINT,
  expectRouterErr,
  localRpc,
  prefundSwapViaStakeFixturesTest,
  prefundWithdrawStakeFixturesTest,
  routerForSwaps,
  STSOL_MINT,
} from "../utils";
import {
  quotePrefundSwapViaStake,
  quotePrefundWithdrawStake,
} from "@sanctumso/sanctum-router";

const STSOL_TOKEN_ACC_NAME = "signer-stsol-token";
const STSOL_EXCEED_WITHDRAW_LAMPORTS_IN_STSOL = 310_355_474_592n;

describe("Lido Test", async () => {
  // PrefundWithdrawStake

  it("lido-prefund-withraw-stake", async () => {
    const q = await prefundWithdrawStakeFixturesTest(
      10_000_000_000n,
      STSOL_TOKEN_ACC_NAME,
    );
    expect(q).toMatchInlineSnapshot(`
      {
        "prefundFee": 1000000000n,
        "quote": {
          "fee": 0n,
          "inp": 10000000000n,
          "out": {
            "staked": 11124056414n,
            "unstaked": 2282880n,
          },
          "vote": "8jxSHbS4qAnh5yueFp4D9ABXubKqMwXqF3HtdzQGuphp",
        },
      }
    `);
  });

  it("lido-prefund-withraw-stake-fails-withdrawal-too-large", async () => {
    const rpc = localRpc();
    const router = await routerForSwaps(rpc, [
      { swap: "prefundWithdrawStake", inp: STSOL_MINT },
    ]);
    await expectRouterErr(
      () =>
        quotePrefundWithdrawStake(router, {
          amt: STSOL_EXCEED_WITHDRAW_LAMPORTS_IN_STSOL,
          inp: STSOL_MINT,
        }),
      "SizeTooLargeErr:LidoError::InvalidAmount",
    );
  });

  it.each([
    { suf: "prefund", amt: 1_000n },
    {
      suf: "splitting-slumdog",
      // more than too-small-for-prefund, but less than what's required for slumdog split
      // and both stakes to be more than stake prog min delegation
      amt: 1_000_000_000n,
    },
  ])(
    "lido-prefund-withdraw-stake-fails-withdrawal-too-small-for-$suf",
    async ({ amt }) => {
      const rpc = localRpc();
      const router = await routerForSwaps(rpc, [
        { swap: "prefundWithdrawStake", inp: STSOL_MINT },
      ]);
      await expectRouterErr(
        () =>
          quotePrefundWithdrawStake(router, {
            amt,
            inp: STSOL_MINT,
          }),
        "SizeTooSmallErr:withdrawn stake too small",
      );
    },
  );

  // PrefundSwapViaStake

  it("lido-prefund-swap-via-stake-into-reserve", async () => {
    const q = await prefundSwapViaStakeFixturesTest(10_000_000_000n, {
      inp: STSOL_TOKEN_ACC_NAME,
      out: "signer-wsol-token",
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "prefundFee": 1000000000n,
        "quote": {
          "quote": {
            "bridge": {
              "lamports": {
                "staked": 11124056414n,
                "unstaked": 2282880n,
              },
              "vote": "8jxSHbS4qAnh5yueFp4D9ABXubKqMwXqF3HtdzQGuphp",
            },
            "inp": 10000000000n,
            "inpFee": 0n,
            "out": 11115070700n,
            "outFee": 11268594n,
          },
          "routerFee": 0n,
        },
      }
    `);
  });

  it("lido-prefund-swap-via-stake-into-reserve-use-bridge-vote", async () => {
    const q = await prefundSwapViaStakeFixturesTest(
      10_000_000_000n,
      {
        inp: STSOL_TOKEN_ACC_NAME,
        out: "signer-wsol-token",
      },
      {
        useBridgeVote: true,
      },
    );
    expect(q).toMatchInlineSnapshot(`
      {
        "prefundFee": 1000000000n,
        "quote": {
          "quote": {
            "bridge": {
              "lamports": {
                "staked": 11124056414n,
                "unstaked": 2282880n,
              },
              "vote": "8jxSHbS4qAnh5yueFp4D9ABXubKqMwXqF3HtdzQGuphp",
            },
            "inp": 10000000000n,
            "inpFee": 0n,
            "out": 11115070700n,
            "outFee": 11268594n,
          },
          "routerFee": 0n,
        },
      }
    `);
  });

  it("lido-prefund-swap-via-stake-into-marinade", async () => {
    const q = await prefundSwapViaStakeFixturesTest(10_000_000_000n, {
      inp: STSOL_TOKEN_ACC_NAME,
      out: "signer-msol-token",
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "prefundFee": 1000000000n,
        "quote": {
          "quote": {
            "bridge": {
              "lamports": {
                "staked": 11124056414n,
                "unstaked": 2282880n,
              },
              "vote": "8jxSHbS4qAnh5yueFp4D9ABXubKqMwXqF3HtdzQGuphp",
            },
            "inp": 10000000000n,
            "inpFee": 0n,
            "out": 8624331770n,
            "outFee": 0n,
          },
          "routerFee": 8632964n,
        },
      }
    `);
  });

  it("lido-prefund-swap-via-stake-into-marinade-use-bridge-vote", async () => {
    const q = await prefundSwapViaStakeFixturesTest(
      10_000_000_000n,
      {
        inp: STSOL_TOKEN_ACC_NAME,
        out: "signer-msol-token",
      },
      {
        useBridgeVote: true,
      },
    );
    expect(q).toMatchInlineSnapshot(`
      {
        "prefundFee": 1000000000n,
        "quote": {
          "quote": {
            "bridge": {
              "lamports": {
                "staked": 11124056414n,
                "unstaked": 2282880n,
              },
              "vote": "8jxSHbS4qAnh5yueFp4D9ABXubKqMwXqF3HtdzQGuphp",
            },
            "inp": 10000000000n,
            "inpFee": 0n,
            "out": 8624331770n,
            "outFee": 0n,
          },
          "routerFee": 8632964n,
        },
      }
    `);
  });

  it("lido-prefund-swap-via-stake-into-spl-bsol", async () => {
    const q = await prefundSwapViaStakeFixturesTest(10_000_000_000n, {
      inp: STSOL_TOKEN_ACC_NAME,
      out: "signer-bsol-token",
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "prefundFee": 1000000000n,
        "quote": {
          "quote": {
            "bridge": {
              "lamports": {
                "staked": 11124056414n,
                "unstaked": 2282880n,
              },
              "vote": "8jxSHbS4qAnh5yueFp4D9ABXubKqMwXqF3HtdzQGuphp",
            },
            "inp": 10000000000n,
            "inpFee": 0n,
            "out": 9069043074n,
            "outFee": 9086836n,
          },
          "routerFee": 9078121n,
        },
      }
    `);
  });

  it("lido-prefund-swap-via-stake-into-spl-bsol-use-bridge-vote", async () => {
    const q = await prefundSwapViaStakeFixturesTest(
      10_000_000_000n,
      {
        inp: STSOL_TOKEN_ACC_NAME,
        out: "signer-bsol-token",
      },
      { useBridgeVote: true },
    );
    expect(q).toMatchInlineSnapshot(`
      {
        "prefundFee": 1000000000n,
        "quote": {
          "quote": {
            "bridge": {
              "lamports": {
                "staked": 11124056414n,
                "unstaked": 2282880n,
              },
              "vote": "8jxSHbS4qAnh5yueFp4D9ABXubKqMwXqF3HtdzQGuphp",
            },
            "inp": 10000000000n,
            "inpFee": 0n,
            "out": 9069043074n,
            "outFee": 9086836n,
          },
          "routerFee": 9078121n,
        },
      }
    `);
  });

  it("lido-prefund-swap-via-stake-fails-withdrawal-too-large", async () => {
    const rpc = localRpc();
    const router = await routerForSwaps(rpc, [
      { swap: "prefundSwapViaStake", inp: STSOL_MINT, out: BSOL_MINT },
    ]);
    await expectRouterErr(
      () =>
        quotePrefundSwapViaStake(router, {
          amt: STSOL_EXCEED_WITHDRAW_LAMPORTS_IN_STSOL,
          inp: STSOL_MINT,
          out: BSOL_MINT,
        }),
      "SizeTooLargeErr:LidoError::InvalidAmount",
    );
  });

  it.each([
    { suf: "prefund", amt: 1_000n },
    {
      suf: "splitting-slumdog",
      // more than too-small-for-prefund, but less than what's required for slumdog split
      // and both stakes to be more than stake prog min delegation
      amt: 1_000_000_000n,
    },
  ])(
    "lido-prefund-swap-via-stake-fails-withdrawal-too-small-for-$suf",
    async ({ amt }) => {
      const rpc = localRpc();
      const router = await routerForSwaps(rpc, [
        { swap: "prefundSwapViaStake", inp: STSOL_MINT, out: BSOL_MINT },
      ]);
      await expectRouterErr(
        () =>
          quotePrefundSwapViaStake(router, {
            amt,
            inp: STSOL_MINT,
            out: BSOL_MINT,
          }),
        "SizeTooSmallErr:withdrawn stake too small",
      );
    },
  );
});
