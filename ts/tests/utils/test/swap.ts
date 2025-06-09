/**
 * Common test utils for token -> token swaps
 * (StakeWrappedSol, PrefundSwapViaStake, WithdrawWrappedSol)
 */

import {
  findFeeTokenAccountPda,
  type Instruction,
  type SwapParams,
  type TokenQuote,
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
import { ixToSimTx } from "../tx";
import { txSimParams } from "./common";

function discmQuote(
  quote: TokenQuote | TokenQuoteWithRouterFee
): quote is TokenQuote {
  return (quote as any).routerFee == null;
}

export async function simTokenSwapAssertQuoteMatches(
  rpc: Rpc<SolanaRpcApi>,
  quote: TokenQuote | TokenQuoteWithRouterFee,
  {
    amount,
    sourceTokenAccount,
    destinationTokenAccount,
    tokenTransferAuthority,
    destinationMint,
  }: SwapParams,
  ix: Instruction
) {
  const [{ inAmount, outAmount }, routerFee] = discmQuote(quote)
    ? [quote, 0n]
    : [quote.quote, quote.routerFee];

  expect(inAmount).toStrictEqual(amount);

  // `addresses` layout:
  // - sourceTokenAccount
  // - destinationTokenAccount
  // - router fee token account
  const addresses = mapTup(
    [
      sourceTokenAccount,
      destinationTokenAccount,
      findFeeTokenAccountPda(destinationMint)[0],
    ],
    address
  );

  const befSwap = await fetchAccountMap(rpc, addresses);
  const [
    sourceTokenAccountBalanceBef,
    destinationTokenAccountBalanceBef,
    feeTokenAccountBalanceBef,
  ] = mapTup(addresses, (addr) => tokenAccBalance(befSwap.get(addr)!.data));

  const tx = ixToSimTx(address(tokenTransferAuthority), ix);
  const {
    value: { err, accounts: aftSwap, logs },
  } = await rpc.simulateTransaction(tx, txSimParams(addresses)).send();

  const debugMsg = `tx: ${tx}\nlogs:\n` + (logs ?? []).join("\n") + "\n";
  expect(err, debugMsg).toBeNull();

  const [
    sourceTokenAccountBalanceAft,
    destinationTokenAccountBalanceAft,
    feeTokenAccountBalanceAft,
  ] = mapTup([0, 1, 2], (i) =>
    tokenAccBalance(
      new Uint8Array(getBase64Encoder().encode(aftSwap[i]!.data[0]))
    )
  );

  expect(sourceTokenAccountBalanceBef - sourceTokenAccountBalanceAft).toEqual(
    inAmount
  );
  expect(
    destinationTokenAccountBalanceAft - destinationTokenAccountBalanceBef
  ).toEqual(outAmount);
  expect(feeTokenAccountBalanceAft - feeTokenAccountBalanceBef).toEqual(
    routerFee
  );
}
