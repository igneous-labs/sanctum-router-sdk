import type { Instruction } from "@sanctumso/sanctum-router";
import {
  appendTransactionMessageInstructions,
  blockhash,
  compileTransaction,
  createTransactionMessage,
  getBase64EncodedWireTransaction,
  pipe,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  type Address,
  type Base64EncodedWireTransaction,
  type IInstruction,
} from "@solana/kit";
import { getSetComputeUnitLimitInstruction } from "@solana-program/compute-budget";

export function ixToSimTx(
  payer: Address,
  ix: Instruction
): Base64EncodedWireTransaction {
  // Examples of very expensive transactions that require >200k default CUs
  // - (Prefund)SwapViaStake
  // - (Prefund)WithdrawStake for lido
  const cuLimitIx = getSetComputeUnitLimitInstruction({ units: 1_500_000 });

  return pipe(
    createTransactionMessage({ version: 0 }),
    (txm) =>
      appendTransactionMessageInstructions(
        [cuLimitIx, ix as unknown as IInstruction],
        txm
      ),
    (txm) => setTransactionMessageFeePayer(payer, txm),
    (txm) =>
      setTransactionMessageLifetimeUsingBlockhash(
        {
          blockhash: blockhash("11111111111111111111111111111111"),
          lastValidBlockHeight: 0n,
        },
        txm
      ),
    compileTransaction,
    getBase64EncodedWireTransaction
  );
}

export function txSimParams(addresses: Address[]) {
  return {
    accounts: {
      addresses,
      encoding: "base64",
    },
    encoding: "base64",
    sigVerify: false,
    replaceRecentBlockhash: true,
  } as const;
}
