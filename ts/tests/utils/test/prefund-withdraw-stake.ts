import {
  findBridgeStakeAccPda,
  prefundWithdrawStakeIx,
  quotePrefundWithdrawStake,
  type Instruction,
  type PrefundWithdrawStakeQuote,
  type WithdrawStakeSwapParams,
} from "@sanctumso/sanctum-router";
import { routerForMints } from "../router";
import { fetchAccountMap, localRpc } from "../rpc";
import { NATIVE_MINT, testFixturesTokenAcc, tokenAccBalance } from "../token";
import {
  address,
  getBase64Encoder,
  lamports,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";
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
  const { addr: inpTokenAcc, owner: signer } =
    testFixturesTokenAcc(inpTokenAccName);
  const rpc = localRpc();

  // TODO: this API is very ass bec we need to remember to include NATIVE_MINT
  // as part of the array or else we will get ReserveError(NotEnoughLiquidity)
  // when quoting because reserves' sol reserves acc is not fetched.
  //
  // GH issue #18 fine-grained updates should aim to solve this
  const router = await routerForMints(rpc, [inpMint, NATIVE_MINT]);

  const quote = quotePrefundWithdrawStake(router, {
    amt,
    inp: inpMint,
    out: outVote,
  });
  const params: WithdrawStakeSwapParams = {
    amt,
    inp: inpMint,
    out: quote.quote.vote,
    signerInp: inpTokenAcc,
    bridgeStakeSeed: BRIDGE_STAKE_SEED,
    signer,
  };

  const ix = prefundWithdrawStakeIx(router, params);

  await simPrefundWithdrawStakeAssertQuoteMatches(rpc, quote, params, ix);
}

async function simPrefundWithdrawStakeAssertQuoteMatches(
  rpc: Rpc<SolanaRpcApi>,
  {
    quote: {
      inp,
      out: { staked, unstaked },
    },
  }: PrefundWithdrawStakeQuote,
  { signerInp, signer }: WithdrawStakeSwapParams,
  ix: Instruction
) {
  // `addresses` layout:
  // - signer input token acc
  // - output bridge stake acc
  const addresses = [
    address(signerInp),
    address(findBridgeStakeAccPda(signer, BRIDGE_STAKE_SEED)[0]),
  ];

  const befSwap = await fetchAccountMap(rpc, addresses);
  const inpTokenAccBalBef = tokenAccBalance(befSwap.get(signerInp)!.data);

  const tx = ixToSimTx(address(signer), ix);
  const {
    value: { err, accounts: aftSwap, logs },
  } = await rpc.simulateTransaction(tx, txSimParams(addresses)).send();

  const debugMsg = `tx: ${tx}\nlogs:\n` + (logs ?? []).join("\n") + "\n";
  expect(err, debugMsg).toBeNull();

  const inpTokenAccBalAft = tokenAccBalance(
    new Uint8Array(getBase64Encoder().encode(aftSwap[0]!.data[0]))
  );
  const bridgeStakeAccBalAft = aftSwap[1]!.lamports;

  expect(inpTokenAccBalBef - inpTokenAccBalAft).toEqual(inp);
  expect(bridgeStakeAccBalAft).toEqual(lamports(staked + unstaked));
}
