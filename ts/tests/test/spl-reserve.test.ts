import { describe, it, assert } from "vitest";
import {
  depositSolFixturesTest,
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

describe("Deposit SOL Test", async () => {
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

  it.sequential("spl-stake-pool-deposit-stake", async () => {
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

    let signerPicoToken = readTestFixturesJsonFile("signer-pico-token");
    let picoVote = readTestFixturesJsonFile("picosol-vote-account");
    let depositStakeAccount = readTestFixturesJsonFile("deposit-stake");
    let picoVault = readTestFixturesJsonFile("pico-vault");

    const [
      signerTokenBalanceBefore,
      picoVaultBalanceBefore,
      managerTokenBalanceBefore,
    ] = await Promise.all(
      mapTup(
        [
          signerPicoToken.pubkey,
          picoVault.pubkey,
          address(stakePool.managerFeeAccount),
        ],
        async (a) =>
          BigInt(
            (
              await rpcClient.getTokenAccountBalance(a).send()
            ).value.amount
          )
      )
    );

    const splLsts: SplPoolAccounts[] = [
      {
        pool: stakePoolJson.pubkey,
        validatorList: validatorListJson.pubkey,
      },
    ];

    let initAccounts = getInitAccounts(splLsts);
    let accounts = await fetchAccountMap(rpcClient, initAccounts);

    let sanctumRouter = fromFetchedAccounts(splLsts, accounts, BigInt(0));

    let accountsToUpdate = getAccountsToUpdate(sanctumRouter, [
      stakePool.poolMint,
    ]);
    let accountsToUpdateMap = await fetchAccountMap(
      rpcClient,
      accountsToUpdate
    );
    update(sanctumRouter, [stakePool.poolMint], accountsToUpdateMap);

    let quote = getDepositStakeQuote(sanctumRouter, {
      validatorVote: picoVote.pubkey,
      outputMint: stakePool.poolMint,
      stakeAccountLamports: {
        staked: BigInt(0),
        unstaked: BigInt(1002282880),
      },
    });

    let ix = getDepositStakeIx(sanctumRouter, {
      amount: BigInt(0),
      source: picoVote.pubkey,
      destinationMint: stakePool.poolMint,
      sourceTokenAccount: depositStakeAccount.pubkey,
      destinationTokenAccount: signerPicoToken.pubkey,
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
      picoVaultBalanceAfter,
      managerTokenBalanceAfter,
    ] = await Promise.all(
      mapTup(
        [
          signerPicoToken.pubkey,
          picoVault.pubkey,
          address(stakePool.managerFeeAccount),
        ],
        async (a) =>
          BigInt(
            (
              await rpcClient.getTokenAccountBalance(a).send()
            ).value.amount
          )
      )
    );

    const globalFee = picoVaultBalanceAfter - picoVaultBalanceBefore;

    // Refetching stake pool to get updated state
    const changedStakePool = await rpcClient
      .getAccountInfo(address(stakePoolJson.pubkey), {
        encoding: "base64",
      })
      .send();
    const newStakePoolData = new Uint8Array(
      getBase64Encoder().encode(changedStakePool.value!.data[0])
    );
    const newStakePoolHandle = deserStakePool(newStakePoolData);
    const newStakePool = getStakePool(newStakePoolHandle);

    assert.strictEqual(
      newStakePool.poolTokenSupply - stakePool.poolTokenSupply,
      quote!.tokensOut + quote!.feeAmount
    );
    assert.strictEqual(
      newStakePool.totalLamports - stakePool.totalLamports,
      BigInt(depositStakeAccount.account.lamports)
    );

    assert.strictEqual(
      managerTokenBalanceAfter - managerTokenBalanceBefore,
      quote!.feeAmount - globalFee
    );
    assert.strictEqual(
      signerTokenBalanceAfter - signerTokenBalanceBefore,
      quote!.tokensOut
    );
  });

  // Reserve test is present here because of the WsolVault fixture
  it.sequential("reserve-deposit-stake", async () => {
    let depositStakeAccount = readTestFixturesJsonFile("reserve-stake-account");
    let signerWsolToken = readTestFixturesJsonFile("reserve-signer-wsol-token");
    let wsolVault = readTestFixturesJsonFile("wsol-vault");
    let picoVote = readTestFixturesJsonFile("picosol-vote-account");
    let picoMint = readTestFixturesJsonFile("pico-sol-mint");
    let protocolFeeDest = readTestFixturesJsonFile("reserve-protocol-vault");
    let poolSolReserves = readTestFixturesJsonFile("reserve-pool-sol-reserves");

    let initAccounts = getInitAccounts([]);
    let accounts = await fetchAccountMap(rpcClient, initAccounts);

    let sanctumRouter = fromFetchedAccounts([], accounts, BigInt(0));

    let accountsToUpdate = getAccountsToUpdate(sanctumRouter, [NATIVE_MINT]);
    let accountsToUpdateMap = await fetchAccountMap(
      rpcClient,
      accountsToUpdate
    );

    update(sanctumRouter, [NATIVE_MINT], accountsToUpdateMap);

    let quote = getDepositStakeQuote(sanctumRouter, {
      validatorVote: picoVote.pubkey,
      outputMint: NATIVE_MINT,
      stakeAccountLamports: {
        staked: BigInt(0),
        unstaked: BigInt(1002282880),
      },
    });

    const [signerTokenBalanceBefore, wsolVaultBalanceBefore] =
      await Promise.all(
        mapTup([signerWsolToken.pubkey, wsolVault.pubkey], async (a) =>
          BigInt(
            (
              await rpcClient.getTokenAccountBalance(a).send()
            ).value.amount
          )
        )
      );

    const [protocolFeeDestBalanceBefore, poolSolReservesBalanaceBefore] =
      await Promise.all(
        mapTup([protocolFeeDest.pubkey, poolSolReserves.pubkey], async (a) =>
          BigInt(
            (
              await rpcClient
                .getAccountInfo(a as Address, { encoding: "base64" })
                .send()
            ).value!.lamports
          )
        )
      );

    let ix = getDepositStakeIx(sanctumRouter, {
      amount: BigInt(0),
      source: picoMint.pubkey,
      destinationMint: NATIVE_MINT,
      sourceTokenAccount: depositStakeAccount.pubkey,
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

    const [signerTokenBalanceAfter, wsolVaultBalanceAfter] = await Promise.all(
      mapTup([signerWsolToken.pubkey, wsolVault.pubkey], async (a) =>
        BigInt((await rpcClient.getTokenAccountBalance(a).send()).value.amount)
      )
    );

    const [protocolFeeDestBalanceAfter, poolSolReservesBalanaceAfter] =
      await Promise.all(
        mapTup([protocolFeeDest.pubkey, poolSolReserves.pubkey], async (a) =>
          BigInt(
            (
              await rpcClient
                .getAccountInfo(a as Address, { encoding: "base64" })
                .send()
            ).value!.lamports
          )
        )
      );

    const protocolFees =
      protocolFeeDestBalanceAfter - protocolFeeDestBalanceBefore;

    const poolSolReservesDelta =
      poolSolReservesBalanaceBefore - poolSolReservesBalanaceAfter;

    const feesForPoolSolReserves =
      poolSolReservesDelta - quote!.tokensOut - BigInt(1002240);

    // No global fees
    assert.strictEqual(wsolVaultBalanceAfter, wsolVaultBalanceBefore);
    assert.strictEqual(
      signerTokenBalanceAfter - signerTokenBalanceBefore,
      quote!.tokensOut
    );
    // No referrer fees
    assert.strictEqual(protocolFees, feesForPoolSolReserves);
    assert.strictEqual(protocolFees + feesForPoolSolReserves, quote!.feeAmount);
    assert.strictEqual(
      poolSolReservesDelta,
      quote!.tokensOut +
        protocolFees +
        // Stake account record rent
        BigInt(1002240)
    );
  });
});
