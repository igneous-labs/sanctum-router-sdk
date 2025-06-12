import {
  findFeeTokenAccountPda,
  getDepositStakeIx,
  getPrefundWithdrawStakeQuote,
  type DepositStakeQuoteWithRouterFee,
  type DepositStakeSwapParams,
  type Instruction,
  type WithdrawStakeSwapParams,
} from "@sanctumso/sanctum-router";
import { routerForMints } from "../router";
import { fetchAccountMap, localRpc } from "../rpc";
import { testFixturesStakeAcc } from "../stake";
import { testFixturesTokenAcc, tokenAccBalance } from "../token";
import {
  address,
  getBase64Encoder,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";
import { mapTup } from "../ops";
import { ixToSimTx, txSimParams } from "../tx";
import { expect } from "vitest";

// Assume bridge stake seed 0 is always unsed
const BRIDGE_STAKE_SEED = 0;

export async function prefundWithdrawStakeFixturesTest(
  amt: bigint,
  inpMint: string,
  inpTokenAccName: string,
  outVote?: string | undefined
) {
  const { addr: inpTokenAcc } = testFixturesTokenAcc(inpTokenAccName);
  const rpc = localRpc();
  const router = await routerForMints(rpc, [inpMint]);

  const quote = getPrefundWithdrawStakeQuote(router, {
    amt,
    inpMint,
    outVote,
  })!;
  const params: WithdrawStakeSwapParams = {
    inp: inpMint,
    out: outVote,
    signerInp: inpStakeAcc,
    signerOut: outTokenAcc,
    signer: withdrawer,
  };

  const ix = getDepositStakeIx(router, params);

  await simPrefundWithdrawStakeAssertQuoteMatches(rpc, quote, params, ix);
}

async function simPrefundWithdrawStakeAssertQuoteMatches(
  rpc: Rpc<SolanaRpcApi>,
  { quote: { out }, routerFee }: DepositStakeQuoteWithRouterFee,
  { out: outMint, signerOut, signer }: DepositStakeSwapParams,
  ix: Instruction
) {
  // `addresses` layout:
  // - signerOut
  // - router fee token account
  const addresses = [
    address(signerOut),
    address(findFeeTokenAccountPda(outMint)[0]),
  ];

  const befSwap = await fetchAccountMap(rpc, addresses);
  const [outTokenAccBalBef, feeTokenAccBalBef] = mapTup(addresses, (addr) =>
    tokenAccBalance(befSwap.get(addr)!.data)
  );

  const tx = ixToSimTx(address(signer), ix);
  const {
    value: { err, accounts: aftSwap, logs },
  } = await rpc.simulateTransaction(tx, txSimParams(addresses)).send();

  const debugMsg = `tx: ${tx}\nlogs:\n` + (logs ?? []).join("\n") + "\n";
  expect(err, debugMsg).toBeNull();

  const [outTokenAccBalAft, feeTokenAccBalAft] = mapTup([0, 1], (i) =>
    tokenAccBalance(
      new Uint8Array(getBase64Encoder().encode(aftSwap[i]!.data[0]))
    )
  );

  expect(outTokenAccBalAft - outTokenAccBalBef).toEqual(out);
  expect(feeTokenAccBalAft - feeTokenAccBalBef).toEqual(routerFee);
}
