import {
  fromFetchedAccounts,
  getAccountsToUpdate,
  getInitAccounts,
  update,
  type B58PK,
  type SanctumRouterHandle,
  type SplPoolAccounts,
} from "@sanctumso/sanctum-router";
import type { Rpc, SolanaRpcApi } from "@solana/kit";
import { fetchAccountMap } from "./rpc";

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
export async function routerForMints(
  rpc: Rpc<SolanaRpcApi>,
  spls: SplPoolAccounts[],
  mints: B58PK[],
  currEpoch: bigint = 0n
): Promise<SanctumRouterHandle> {
  const initAccounts = getInitAccounts(spls);
  const accounts = await fetchAccountMap(rpc, initAccounts);
  const sanctumRouter = fromFetchedAccounts(spls, accounts, currEpoch);

  const accountsToUpdate = getAccountsToUpdate(sanctumRouter, mints);
  const accountsToUpdateMap = await fetchAccountMap(rpc, accountsToUpdate);
  update(sanctumRouter, mints, accountsToUpdateMap);

  return sanctumRouter;
}
