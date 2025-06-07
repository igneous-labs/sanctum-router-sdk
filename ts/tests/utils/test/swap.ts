/**
 * Common test utils for token -> token swaps
 * (StakeWrappedSol, PrefundSwapViaStake, WithdrawWrappedSol)
 */

import type {
  Instruction,
  SwapParams,
  TokenQuote,
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

export async function simTokenSwapAssertQuoteMatches(
  rpc: Rpc<SolanaRpcApi>,
  {
    inAmount,
    outAmount,
    // TODO: need to also assert that the router fee accounts received the correct amount of
    // fees but that would mean modifying the TokenQuote struct def to have fine-grained fee breakdowns
    // of stake pool fees + router fees
    feeAmount: _,
  }: TokenQuote,
  {
    amount,
    sourceTokenAccount,
    destinationTokenAccount,
    tokenTransferAuthority,
  }: SwapParams,
  ix: Instruction
) {
  expect(inAmount).toStrictEqual(amount);

  // `addresses` layout:
  // - sourceTokenAccount
  // - destinationTokenAccount
  const addresses = mapTup(
    [sourceTokenAccount, destinationTokenAccount],
    address
  );

  const befSwap = await fetchAccountMap(rpc, addresses);
  const [sourceTokenAccountBalanceBef, destinationTokenAccountBalanceBef] =
    mapTup(addresses, (addr) => tokenAccBalance(befSwap.get(addr)!.data));

  const tx = ixToSimTx(address(tokenTransferAuthority), ix);
  const {
    value: { err, accounts: aftSwap, logs },
  } = await rpc.simulateTransaction(tx, txSimParams(addresses)).send();

  const debugMsg = `tx: ${tx}\nlogs:\n` + (logs ?? []).join("\n") + "\n";
  expect(err, debugMsg).toBeNull();

  const [sourceTokenAccountBalanceAft, destinationTokenAccountBalanceAft] =
    mapTup([0, 1], (i) =>
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
}
