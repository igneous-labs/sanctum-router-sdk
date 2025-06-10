/**
 * Common test utils for token -> token swaps
 * (StakeWrappedSol, PrefundSwapViaStake, WithdrawWrappedSol)
 */

import {
  findFeeTokenAccountPda,
  type Instruction,
  type TokenSwapParams,
  type TokenQuoteWithRouterFee,
} from "@sanctumso/sanctum-router";
import {
  address,
  getBase64Encoder,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";
import { expect } from "vitest";
import { mapTup } from "../ops";
import { fetchAccountMap } from "../rpc";
import { tokenAccBalance } from "../token";
import { ixToSimTx, txSimParams } from "../tx";

export async function simTokenSwapAssertQuoteMatches(
  rpc: Rpc<SolanaRpcApi>,
  { quote: { inp, out }, routerFee }: TokenQuoteWithRouterFee,
  { out: outMint, amt, signer, signerInp, signerOut }: TokenSwapParams,
  ix: Instruction
) {
  expect(inp).toStrictEqual(amt);

  // `addresses` layout:
  // - signerInp
  // - signerOut
  // - router fee token account
  const addresses = mapTup(
    [signerInp, signerOut, findFeeTokenAccountPda(outMint)[0]],
    address
  );

  const befSwap = await fetchAccountMap(rpc, addresses);
  const [inpTokenAccBalBef, outTokenAccBalBef, feeTokenAccBalBef] = mapTup(
    addresses,
    (addr) => tokenAccBalance(befSwap.get(addr)!.data)
  );

  const tx = ixToSimTx(address(signer), ix);
  const {
    value: { err, accounts: aftSwap, logs },
  } = await rpc.simulateTransaction(tx, txSimParams(addresses)).send();

  const debugMsg = `tx: ${tx}\nlogs:\n` + (logs ?? []).join("\n") + "\n";
  expect(err, debugMsg).toBeNull();

  const [inpTokenAccBalAft, outTokenAccBalAft, feeTokenAccBalAft] = mapTup(
    [0, 1, 2],
    (i) =>
      tokenAccBalance(
        new Uint8Array(getBase64Encoder().encode(aftSwap[i]!.data[0]))
      )
  );

  expect(inpTokenAccBalBef - inpTokenAccBalAft).toEqual(inp);
  expect(outTokenAccBalAft - outTokenAccBalBef).toEqual(out);
  expect(feeTokenAccBalAft - feeTokenAccBalBef).toEqual(routerFee);
}
