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
import { BSOL_ACCS, PICOSOL_ACCS } from "./spl";

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
  mints: B58PK[],
  // TODO: bsol and picosol are currently the only SPL pools being tested.
  // May need to add to this list in the future if we add more.
  spls: SplPoolAccounts[] = [BSOL_ACCS, PICOSOL_ACCS],
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
