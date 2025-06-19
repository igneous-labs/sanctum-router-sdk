import {
  accountsToUpdate,
  init,
  newSanctumRouter,
  update,
  type SanctumRouterHandle,
  type InitMint,
  type SwapMints,
  initSyncEmbed,
} from "@sanctumso/sanctum-router";
import type { Rpc, SolanaRpcApi } from "@solana/kit";
import { fetchAccountMap } from "./rpc";
import { SPL_INIT_HARDCODES } from "./spl";
import { NATIVE_MINT } from "./token";

/**
 * Initializes, updates and returns `SanctumRouterHandle` that is ready for quoting
 * and trading between `mints`
 *
 * Assumes `SanctumRouterHandle` only needs to do a single update for the given mints
 * before it is ready for use.
 *
 * @param rpc
 * @param spls
 * @param mints
 * @param currEpoch
 */
export async function routerForSwaps(
  rpc: Rpc<SolanaRpcApi>,
  swapMints: SwapMints[]
): Promise<SanctumRouterHandle> {
  initSyncEmbed();

  const sanctumRouter = newSanctumRouter();

  const initMints: InitMint[] = swapMints
    .flatMap((swapMint) => {
      switch (swapMint.swap) {
        case "depositSol":
          return [swapMint.out];
        case "depositStake":
          return [swapMint.out];
        case "withdrawSol":
          return [swapMint.inp];
        case "prefundWithdrawStake":
          return [swapMint.inp, NATIVE_MINT];
        case "prefundSwapViaStake":
          return [swapMint.inp, swapMint.out, NATIVE_MINT];
      }
    })
    .map((mint) => {
      const splInitOpt = SPL_INIT_HARDCODES[mint];
      if (splInitOpt) {
        return { mint, init: { pool: "spl", ...splInitOpt } };
      } else {
        return { mint };
      }
    });
  init(sanctumRouter, initMints);

  const accs = accountsToUpdate(sanctumRouter, swapMints);
  const accountsToUpdateMap = await fetchAccountMap(rpc, accs);
  update(sanctumRouter, swapMints, accountsToUpdateMap);

  return sanctumRouter;
}
