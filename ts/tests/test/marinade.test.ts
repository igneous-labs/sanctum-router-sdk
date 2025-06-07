import { describe, it, assert } from "vitest";
import {
  depositSolFixturesTest,
  fetchAccountMap,
  mapTup,
  MSOL_MINT,
  readTestFixturesJsonFile,
  readTestFixturesKeypair,
} from "../utils";
import {
  fromFetchedAccounts,
  getAccountsToUpdate,
  getDepositStakeIx,
  getDepositStakeQuote,
  getInitAccounts,
  update,
} from "@sanctumso/sanctum-router";
import {
  appendTransactionMessageInstructions,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createTransactionMessage,
  pipe,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
  type IInstruction,
} from "@solana/kit";

describe("Marinade Test", async () => {
  const rpcClient = createSolanaRpc("http://localhost:8899");
  const rpcClientSubscriptions = createSolanaRpcSubscriptions(
    "ws://localhost:8900"
  );
  const keypair = await readTestFixturesKeypair("signer");

  it("marinade-deposit-sol", async () => {
    await depositSolFixturesTest(1000000n, MSOL_MINT, {
      inp: "marinade-signer-wsol-token",
      out: "signer-msol-token",
    });
  });

  it.sequential("marinade-deposit-stake", async () => {
    let msolMint = readTestFixturesJsonFile("marinade-msol_mint");
    let depositStakeAccount = readTestFixturesJsonFile(
      "marinade_stake_account"
    );
    let signerMsolToken = readTestFixturesJsonFile("signer-msol-token");
    let msolVault = readTestFixturesJsonFile("msol-vault");

    let initAccounts = getInitAccounts([]);
    let accounts = await fetchAccountMap(rpcClient, initAccounts);

    let sanctumRouter = fromFetchedAccounts([], accounts, BigInt(0));

    let accountsToUpdate = getAccountsToUpdate(sanctumRouter, [
      msolMint.pubkey,
    ]);
    let accountsToUpdateMap = await fetchAccountMap(
      rpcClient,
      accountsToUpdate
    );
    update(sanctumRouter, [msolMint.pubkey], accountsToUpdateMap);

    let quote = getDepositStakeQuote(sanctumRouter, {
      validatorVote: "BLADE1qNA1uNjRgER6DtUFf7FU3c1TWLLdpPeEcKatZ2",
      outputMint: msolMint.pubkey,
      stakeAccountLamports: {
        staked: BigInt(6666963148180),
        unstaked: BigInt(2282880),
      },
    });

    const [signerTokenBalanceBefore, msolVaultBalanceBefore] =
      await Promise.all(
        mapTup([signerMsolToken.pubkey, msolVault.pubkey], async (a) =>
          BigInt(
            (
              await rpcClient.getTokenAccountBalance(a).send()
            ).value.amount
          )
        )
      );

    let ix = getDepositStakeIx(sanctumRouter, {
      amount: BigInt(0),
      source: "BLADE1qNA1uNjRgER6DtUFf7FU3c1TWLLdpPeEcKatZ2",
      destinationMint: msolMint.pubkey,
      sourceTokenAccount: depositStakeAccount.pubkey,
      destinationTokenAccount: signerMsolToken.pubkey,
      tokenTransferAuthority: keypair.address,
    }) as unknown as IInstruction;

    const { value: blockhash } = await rpcClient.getLatestBlockhash().send();

    const tx = pipe(
      createTransactionMessage({
        version: 0,
      }),
      (txm) => appendTransactionMessageInstructions([ix], txm),
      (txm) => setTransactionMessageFeePayerSigner(keypair, txm),
      (txm) => setTransactionMessageLifetimeUsingBlockhash(blockhash, txm)
    );

    const signedTx = await signTransactionMessageWithSigners(tx);
    const sendAndConfirmTx = sendAndConfirmTransactionFactory({
      rpc: rpcClient,
      rpcSubscriptions: rpcClientSubscriptions,
    });
    await sendAndConfirmTx(signedTx, {
      commitment: "confirmed",
    });

    const [signerTokenBalanceAfter, msolVaultBalanceAfter] = await Promise.all(
      mapTup([signerMsolToken.pubkey, msolVault.pubkey], async (a) =>
        BigInt((await rpcClient.getTokenAccountBalance(a).send()).value.amount)
      )
    );

    assert.strictEqual(
      msolVaultBalanceAfter - msolVaultBalanceBefore,
      quote!.feeAmount
    );

    assert.strictEqual(
      signerTokenBalanceAfter - signerTokenBalanceBefore,
      quote!.tokensOut
    );
  });
});
