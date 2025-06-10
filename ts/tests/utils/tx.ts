import type { Instruction } from "@sanctumso/sanctum-router";
import {
  appendTransactionMessageInstruction,
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

export function ixToSimTx(
  payer: Address,
  ix: Instruction
): Base64EncodedWireTransaction {
  return pipe(
    createTransactionMessage({ version: 0 }),
    (txm) =>
      appendTransactionMessageInstruction(ix as unknown as IInstruction, txm),
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
