import { describe, expect, it } from "vitest";
import {
  depositSolFixturesTest,
  depositStakeFixturesTest,
  expectRouterErr,
  localRpc,
  PICOSOL_MINT,
  prefundSwapViaStakeFixturesTest,
  prefundWithdrawStakeFixturesTest,
  routerForSwaps,
  withdrawSolFixturesTest,
} from "../utils";
import {
  quotePrefundWithdrawStake,
  quoteWithdrawSol,
} from "@sanctumso/sanctum-router";

// picsol validator list fixtures:
// - vsi_active_lamports=210_425__790_541_328
// - mint_supply=108_350_488_973_931
// - total_lamports=128_350__525_083_404
// - stake_withdrawal_fee=0
// (yes i know the numbers dont add up, see test fixtures README)

const PICOSOL_TOKEN_ACC_NAME = "signer-picosol-token";
const PICOSOL_EXCEED_SOL_WITHDRAW = 3_000_613_461_708n;
const PICOSOL_EXCEED_STAKE_WITHDRAW = 210_425_790_541_328n;

describe("SPL Test", async () => {
  // DepositSol
  it("spl-picosol-deposit-sol", async () => {
    const q = await depositSolFixturesTest(1000000n, {
      inp: "signer-wsol-token",
      out: PICOSOL_TOKEN_ACC_NAME,
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "quote": {
          "fee": 845n,
          "inp": 1000000n,
          "out": 843331n,
        },
        "routerFee": 0n,
      }
    `);
  });

  // WithdrawSol
  it("spl-picosol-withdraw-sol", async () => {
    const q = await withdrawSolFixturesTest(1000000n, {
      inp: PICOSOL_TOKEN_ACC_NAME,
      out: "signer-wsol-token",
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "quote": {
          "fee": 1000n,
          "inp": 1000000n,
          "out": 1183283n,
        },
        "routerFee": 118n,
      }
    `);
  });

  it("spl-picosol-withdraw-sol-fails-withdrawal-too-large", async () => {
    const rpc = localRpc();
    const router = await routerForSwaps(rpc, [
      { swap: "withdrawSol", inp: PICOSOL_MINT },
    ]);
    await expectRouterErr(
      () =>
        quoteWithdrawSol(router, {
          amt: PICOSOL_EXCEED_SOL_WITHDRAW,
          inp: PICOSOL_MINT,
        }),
      "SizeTooLargeErr:SplStakePoolError::SolWithdrawalTooLarge",
    );
  });

  // DepositStake
  it("spl-picosol-deposit-stake", async () => {
    const q = await depositStakeFixturesTest({
      inp: "picosol-deposit-stake",
      out: PICOSOL_TOKEN_ACC_NAME,
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "quote": {
          "fee": 1928n,
          "inp": {
            "staked": 1000000000n,
            "unstaked": 2282880n,
          },
          "out": 845255843n,
          "vote": "3xjfK9C9YNcta8MvK1US4sQ3bc6DEjoJoR3qLExGf9xE",
        },
        "routerFee": 846101n,
      }
    `);
  });

  // PrefundWithdrawStake
  it("spl-picosol-prefund-withdraw-stake-small", async () => {
    const q = await prefundWithdrawStakeFixturesTest(
      10_000_000_000n,
      PICOSOL_TOKEN_ACC_NAME,
    );
    expect(q).toMatchInlineSnapshot(`
      {
        "prefundFee": 1000000000n,
        "quote": {
          "fee": 0n,
          "inp": 10000000000n,
          "out": {
            "staked": 10845860920n,
            "unstaked": 2282880n,
          },
          "vote": "3xjfK9C9YNcta8MvK1US4sQ3bc6DEjoJoR3qLExGf9xE",
        },
      }
    `);
  });

  it("spl-picosol-prefund-withdraw-stake-large", async () => {
    const q = await prefundWithdrawStakeFixturesTest(
      750_000_000_000n,
      PICOSOL_TOKEN_ACC_NAME,
    );
    expect(q).toMatchInlineSnapshot(`
      {
        "prefundFee": 1000000000n,
        "quote": {
          "fee": 0n,
          "inp": 750000000000n,
          "out": {
            "staked": 887439569060n,
            "unstaked": 2282880n,
          },
          "vote": "3xjfK9C9YNcta8MvK1US4sQ3bc6DEjoJoR3qLExGf9xE",
        },
      }
    `);
  });

  it("spl-picosol-quote-prefund-withdraw-stake-fails-withdrawal-too-large", async () => {
    const rpc = localRpc();
    const router = await routerForSwaps(rpc, [
      { swap: "prefundWithdrawStake", inp: PICOSOL_MINT },
    ]);
    await expectRouterErr(
      () =>
        quotePrefundWithdrawStake(router, {
          amt: PICOSOL_EXCEED_STAKE_WITHDRAW,
          inp: PICOSOL_MINT,
        }),
      "SizeTooLargeErr:SplStakePoolError::StakeLamportsNotEqualToMinimum",
    );
  });

  // PrefundSwapViaStake

  it("spl-picosol-prefund-swap-via-stake-into-reserve-small", async () => {
    const q = await prefundSwapViaStakeFixturesTest(10_000_000_000n, {
      inp: PICOSOL_TOKEN_ACC_NAME,
      out: "signer-wsol-token",
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "prefundFee": 1000000000n,
        "quote": {
          "quote": {
            "bridge": {
              "lamports": {
                "staked": 10845860920n,
                "unstaked": 2282880n,
              },
              "vote": "3xjfK9C9YNcta8MvK1US4sQ3bc6DEjoJoR3qLExGf9xE",
            },
            "inp": 10000000000n,
            "inpFee": 0n,
            "out": 10837157540n,
            "outFee": 10986260n,
          },
          "routerFee": 0n,
        },
      }
    `);
  });

  it("spl-picosol-prefund-swap-via-stake-into-reserve-large", async () => {
    const q = await prefundSwapViaStakeFixturesTest(750_000_000_000n, {
      inp: PICOSOL_TOKEN_ACC_NAME,
      out: "signer-wsol-token",
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "prefundFee": 1000000000n,
        "quote": {
          "quote": {
            "bridge": {
              "lamports": {
                "staked": 887439569060n,
                "unstaked": 2282880n,
              },
              "vote": "3xjfK9C9YNcta8MvK1US4sQ3bc6DEjoJoR3qLExGf9xE",
            },
            "inp": 750000000000n,
            "inpFee": 0n,
            "out": 886393186735n,
            "outFee": 1048665205n,
          },
          "routerFee": 0n,
        },
      }
    `);
  });

  it("spl-picosol-prefund-swap-via-stake-into-reserve-use-bridge-vote", async () => {
    const q = await prefundSwapViaStakeFixturesTest(
      10_000_000_000n,
      {
        inp: PICOSOL_TOKEN_ACC_NAME,
        out: "signer-wsol-token",
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
                "staked": 10845860920n,
                "unstaked": 2282880n,
              },
              "vote": "3xjfK9C9YNcta8MvK1US4sQ3bc6DEjoJoR3qLExGf9xE",
            },
            "inp": 10000000000n,
            "inpFee": 0n,
            "out": 10837157540n,
            "outFee": 10986260n,
          },
          "routerFee": 0n,
        },
      }
    `);
  });

  it("spl-picosol-prefund-swap-via-stake-into-marinade-small", async () => {
    const q = await prefundSwapViaStakeFixturesTest(10_000_000_000n, {
      inp: PICOSOL_TOKEN_ACC_NAME,
      out: "signer-msol-token",
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "prefundFee": 1000000000n,
        "quote": {
          "quote": {
            "bridge": {
              "lamports": {
                "staked": 10845860920n,
                "unstaked": 2282880n,
              },
              "vote": "3xjfK9C9YNcta8MvK1US4sQ3bc6DEjoJoR3qLExGf9xE",
            },
            "inp": 10000000000n,
            "inpFee": 0n,
            "out": 8408650534n,
            "outFee": 0n,
          },
          "routerFee": 8417067n,
        },
      }
    `);
  });

  it("spl-picosol-prefund-swap-via-stake-into-marinade-large", async () => {
    const q = await prefundSwapViaStakeFixturesTest(750_000_000_000n, {
      inp: PICOSOL_TOKEN_ACC_NAME,
      out: "signer-msol-token",
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "prefundFee": 1000000000n,
        "quote": {
          "quote": {
            "bridge": {
              "lamports": {
                "staked": 887439569060n,
                "unstaked": 2282880n,
              },
              "vote": "3xjfK9C9YNcta8MvK1US4sQ3bc6DEjoJoR3qLExGf9xE",
            },
            "inp": 750000000000n,
            "inpFee": 0n,
            "out": 688019997797n,
            "outFee": 0n,
          },
          "routerFee": 688708706n,
        },
      }
    `);
  });

  it("spl-picosol-prefund-swap-via-stake-into-marinade-use-bridge-vote", async () => {
    const q = await prefundSwapViaStakeFixturesTest(
      10_000_000_000n,
      {
        inp: PICOSOL_TOKEN_ACC_NAME,
        out: "signer-msol-token",
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
                "staked": 10845860920n,
                "unstaked": 2282880n,
              },
              "vote": "3xjfK9C9YNcta8MvK1US4sQ3bc6DEjoJoR3qLExGf9xE",
            },
            "inp": 10000000000n,
            "inpFee": 0n,
            "out": 8408650534n,
            "outFee": 0n,
          },
          "routerFee": 8417067n,
        },
      }
    `);
  });

  it("spl-picosol-prefund-swap-via-stake-into-spl-bsol-small", async () => {
    const q = await prefundSwapViaStakeFixturesTest(10_000_000_000n, {
      inp: PICOSOL_TOKEN_ACC_NAME,
      out: "signer-bsol-token",
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "prefundFee": 1000000000n,
        "quote": {
          "quote": {
            "bridge": {
              "lamports": {
                "staked": 10845860920n,
                "unstaked": 2282880n,
              },
              "vote": "3xjfK9C9YNcta8MvK1US4sQ3bc6DEjoJoR3qLExGf9xE",
            },
            "inp": 10000000000n,
            "inpFee": 0n,
            "out": 8842286839n,
            "outFee": 8859626n,
          },
          "routerFee": 8851137n,
        },
      }
    `);
  });

  it("spl-picosol-prefund-swap-via-stake-into-spl-bsol-large", async () => {
    const q = await prefundSwapViaStakeFixturesTest(750_000_000_000n, {
      inp: PICOSOL_TOKEN_ACC_NAME,
      out: "signer-bsol-token",
    });
    expect(q).toMatchInlineSnapshot(`
      {
        "prefundFee": 1000000000n,
        "quote": {
          "quote": {
            "bridge": {
              "lamports": {
                "staked": 887439569060n,
                "unstaked": 2282880n,
              },
              "vote": "3xjfK9C9YNcta8MvK1US4sQ3bc6DEjoJoR3qLExGf9xE",
            },
            "inp": 750000000000n,
            "inpFee": 0n,
            "out": 723350946174n,
            "outFee": 724799449n,
          },
          "routerFee": 724075021n,
        },
      }
    `);
  });

  it("spl-picosol-prefund-swap-via-stake-into-spl-bsol-use-bridge-vote", async () => {
    const q = await prefundSwapViaStakeFixturesTest(
      10_000_000_000n,
      {
        inp: PICOSOL_TOKEN_ACC_NAME,
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
                "staked": 10845860920n,
                "unstaked": 2282880n,
              },
              "vote": "3xjfK9C9YNcta8MvK1US4sQ3bc6DEjoJoR3qLExGf9xE",
            },
            "inp": 10000000000n,
            "inpFee": 0n,
            "out": 8842286839n,
            "outFee": 8859626n,
          },
          "routerFee": 8851137n,
        },
      }
    `);
  });
});
