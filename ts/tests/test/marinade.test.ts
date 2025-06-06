import { describe, it, assert } from "vitest";
import {
  fetchAccountMap,
  mapTup,
  MSOL_MINT,
  NATIVE_MINT,
  readTestFixturesJsonFile,
  readTestFixturesKeypair,
} from "./utils";
import {
  deserStakePool,
  fromFetchedAccounts,
  getAccountsToUpdate,
  getDepositSolIx,
  getDepositSolQuote,
  getDepositStakeIx,
  getDepositStakeQuote,
  getInitAccounts,
  getStakePool,
  getWithdrawSolIx,
  getWithdrawSolQuote,
  update,
  type SplPoolAccounts,
} from "@sanctumso/sanctum-router";
import {
  address,
  appendTransactionMessageInstructions,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createTransactionMessage,
  getBase64Encoder,
  pipe,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
  type Address,
  type IInstruction,
} from "@solana/kit";

describe("Marinade Test", async () => {
  const rpcClient = createSolanaRpc("http://localhost:8899");
  const rpcClientSubscriptions = createSolanaRpcSubscriptions(
    "ws://localhost:8900"
  );
  const keypair = await readTestFixturesKeypair("signer");

  it.sequential("marinade-deposit-sol", async () => {
    let signerWsolToken = readTestFixturesJsonFile(
      "marinade-signer-wsol-token"
    );
    let signerMSolToken = readTestFixturesJsonFile("signer-msol-token");
    let initAccounts = getInitAccounts([]);
    let accounts = await fetchAccountMap(rpcClient, initAccounts);

    let sanctumRouter = fromFetchedAccounts([], accounts, BigInt(1));

    let accountsToUpdate = getAccountsToUpdate(sanctumRouter, [MSOL_MINT]);
    let accountsToUpdateMap = await fetchAccountMap(
      rpcClient,
      accountsToUpdate
    );
    update(sanctumRouter, [MSOL_MINT], accountsToUpdateMap);

    const [signerTokenBalanceBefore, signerMsolTokenBalanceBefore] =
      await Promise.all(
        mapTup([signerWsolToken.pubkey, signerMSolToken.pubkey], async (a) =>
          BigInt(
            (
              await rpcClient.getTokenAccountBalance(a).send()
            ).value.amount
          )
        )
      );

    let quote = getDepositSolQuote(sanctumRouter, {
      amount: BigInt(1000000),
      outputMint: MSOL_MINT,
      inputMint: NATIVE_MINT,
    });

    let ix = getDepositSolIx(sanctumRouter, {
      amount: BigInt(1000000),
      source: NATIVE_MINT,
      destinationMint: MSOL_MINT,
      sourceTokenAccount: signerWsolToken.pubkey,
      destinationTokenAccount: signerMSolToken.pubkey,
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

    const [signerTokenBalanceAfter, signerMsolTokenBalanceAfter] =
      await Promise.all(
        mapTup([signerWsolToken.pubkey, signerMSolToken.pubkey], async (a) =>
          BigInt(
            (
              await rpcClient.getTokenAccountBalance(a).send()
            ).value.amount
          )
        )
      );

    assert.strictEqual(
      signerTokenBalanceBefore - signerTokenBalanceAfter,
      quote?.inAmount
    );
    assert.strictEqual(
      signerMsolTokenBalanceAfter - signerMsolTokenBalanceBefore,
      quote?.outAmount
    );
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
