import {
  quoteWithdrawSol,
  withdrawSolIx,
  type WithdrawSolSwapParams,
} from "@sanctumso/sanctum-router";
import { mapTup } from "../ops";
import { routerForMints } from "../router";
import { localRpc } from "../rpc";
import { NATIVE_MINT, testFixturesTokenAcc } from "../token";
import { simTokenSwapAssertQuoteMatches } from "./swap";

export async function withdrawSolFixturesTest(
  amt: bigint,
  mint: string,
  tokenAccFixtures: { inp: string; out: string }
) {
  const { inp: inpTokenAccName, out: outTokenAccName } = tokenAccFixtures;
  const [
    { addr: inpTokenAcc, owner: inpTokenAccOwner },
    { addr: outTokenAcc },
  ] = mapTup([inpTokenAccName, outTokenAccName], testFixturesTokenAcc);
  const rpc = localRpc();
  const router = await routerForMints(rpc, [mint]);

  const quote = quoteWithdrawSol(router, {
    amt,
    inp: mint,
  });
  const params: WithdrawSolSwapParams = {
    amt,
    signerInp: inpTokenAcc,
    signerOut: outTokenAcc,
    signer: inpTokenAccOwner,
    inp: mint,
  };
  const ix = withdrawSolIx(router, params);

  await simTokenSwapAssertQuoteMatches(
    rpc,
    quote,
    { ...params, out: NATIVE_MINT },
    ix
  );
}
