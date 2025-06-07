import { describe, it, assert } from "vitest";
import {
  depositSolFixturesTest,
  depositStakeFixturesTest,
  fetchAccountMap,
  mapTup,
  NATIVE_MINT,
  PICOSOL_MINT,
  readTestFixturesJsonFile,
  readTestFixturesKeypair,
} from "../utils";
import {
  deserStakePool,
  fromFetchedAccounts,
  getAccountsToUpdate,
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
  pipe,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
  type Address,
  type IInstruction,
} from "@solana/kit";

describe("SPL Test", async () => {
  const rpcClient = createSolanaRpc("http://localhost:8899");
  const rpcClientSubscriptions = createSolanaRpcSubscriptions(
    "ws://localhost:8900"
  );
  const keypair = await readTestFixturesKeypair("signer");

  it("spl-stake-pool-deposit-sol", async () => {
    await depositSolFixturesTest(1000000n, PICOSOL_MINT, {
      inp: "spl-signer-wsol-token",
      out: "signer-pico-token",
    });
  });

  it.sequential("spl-stake-pool-withdraw-sol", async () => {
    const validatorListJson = readTestFixturesJsonFile(
      "pico-sol-validator-list"
    );
    const stakePoolJson = readTestFixturesJsonFile("pico-sol-stake-pool");
    const stakePoolInfo = await rpcClient
      .getAccountInfo(stakePoolJson.pubkey as Address, {
        encoding: "base64",
      })
      .send();
    const stakePoolData = Buffer.from(stakePoolInfo.value!.data[0], "base64");
    const stakePoolBytes = new Uint8Array(stakePoolData);
    const stakePoolHandle = deserStakePool(stakePoolBytes);
    const stakePool = getStakePool(stakePoolHandle);

    let signerWsolToken = readTestFixturesJsonFile("spl-signer-wsol-token");
    let signerPicoToken = readTestFixturesJsonFile("signer-pico-token");
    let wsolVault = readTestFixturesJsonFile("wsol-vault");

    const splLsts: SplPoolAccounts[] = [
      {
        pool: stakePoolJson.pubkey,
        validatorList: validatorListJson.pubkey,
      },
    ];

    let initAccounts = getInitAccounts(splLsts);
    let accounts = await fetchAccountMap(rpcClient, initAccounts);
    let sanctumRouter = fromFetchedAccounts(splLsts, accounts, BigInt(1));

    let accountsToUpdate = getAccountsToUpdate(sanctumRouter, [
      stakePool.poolMint,
    ]);
    let accountsToUpdateMap = await fetchAccountMap(
      rpcClient,
      accountsToUpdate
    );
    update(sanctumRouter, [stakePool.poolMint], accountsToUpdateMap);

    const [
      signerTokenBalanceBefore,
      signerPicoTokenBalanceBefore,
      managerFeeTokenBalanceBefore,
      wsolVaultBalanceBefore,
    ] = await Promise.all(
      mapTup(
        [
          signerWsolToken.pubkey,
          signerPicoToken.pubkey,
          address(stakePool.managerFeeAccount),
          wsolVault.pubkey,
        ],
        async (a) =>
          BigInt(
            (
              await rpcClient.getTokenAccountBalance(a).send()
            ).value.amount
          )
      )
    );

    const reserveLamportsBefore = BigInt(
      (
        await rpcClient
          .getAccountInfo(address(stakePool.reserveStake), {
            encoding: "base64",
          })
          .send()
      ).value!.lamports
    );

    let quote = getWithdrawSolQuote(sanctumRouter, {
      amount: BigInt(1000000),
      outputMint: NATIVE_MINT,
      inputMint: stakePool.poolMint,
    });

    let ix = getWithdrawSolIx(sanctumRouter, {
      amount: BigInt(1000000),
      source: stakePool.poolMint,
      destinationMint: NATIVE_MINT,
      sourceTokenAccount: signerPicoToken.pubkey,
      destinationTokenAccount: signerWsolToken.pubkey,
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

    const [
      signerTokenBalanceAfter,
      signerPicoTokenBalanceAfter,
      managerFeeTokenBalanceAfter,
      wsolVaultBalanceAfter,
    ] = await Promise.all(
      mapTup(
        [
          signerWsolToken.pubkey,
          signerPicoToken.pubkey,
          address(stakePool.managerFeeAccount),
          wsolVault.pubkey,
        ],
        async (a) =>
          BigInt(
            (
              await rpcClient.getTokenAccountBalance(a).send()
            ).value.amount
          )
      )
    );

    const reserveLamportsAfter = BigInt(
      (
        await rpcClient
          .getAccountInfo(address(stakePool.reserveStake), {
            encoding: "base64",
          })
          .send()
      ).value!.lamports
    );

    const globalFee =
      reserveLamportsBefore - reserveLamportsAfter - quote!.outAmount;

    assert.strictEqual(
      signerTokenBalanceAfter - signerTokenBalanceBefore,
      quote?.outAmount
    );
    assert.strictEqual(
      signerPicoTokenBalanceBefore - signerPicoTokenBalanceAfter,
      quote?.inAmount
    );
    assert.strictEqual(
      managerFeeTokenBalanceAfter - managerFeeTokenBalanceBefore,
      quote!.feeAmount - globalFee
    );
    assert.strictEqual(
      wsolVaultBalanceAfter - wsolVaultBalanceBefore,
      globalFee
    );
  });

  it("spl-stake-pool-deposit-stake", async () => {
    await depositStakeFixturesTest(PICOSOL_MINT, {
      inp: "deposit-stake",
      out: "signer-pico-token",
    });
  });
});
