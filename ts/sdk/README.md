# @sanctumso/sanctum-router

Typescript + WASM SDK for the SPL stake pool program.

## Example Usage

```ts
import {
  createSolanaRpc,
  getBase64Encoder,
  type Address,
  type IInstruction,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";
import {
  accountsToUpdate,
  init,
  newSanctumRouter,
  prefundSwapViaStakeIx,
  quotePrefundSwapViaStake,
  update,
  type InitData,
  type SplPoolAccounts,
} from "@sanctumso/sanctum-router";
import initSdk from "@sanctumso/sanctum-router";

// The SDK needs to be initialized once globally before it can be used (idempotent).
// For nodejs environments, use
// `import { initSyncEmbed } from "@sanctumso/sanctum-router"; initSyncEmbed();`
// instead
await initSdk();

// SPL stake pools (all 3 deploys) must have the following data known beforehand
// and explicitly passed in at initialization time
const PICOSOL_INIT_DATA: InitData = {
  pool: "spl",
  stakePoolAddr: "8Dv3hNYcEWEaa4qVx9BTN1Wfvtha1z8cWDUXb7KVACVe",
  stakePoolProgramAddr: "SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY",
  validatorListAddr: "46A5KjX8J6FAUTXwcE8iJkmM7igK3v8vy1MD74cZNWVE",
  reserveStakeAddr: "2ArodFTZhNqVWJT92qEGDxigAvouSo1kfgfEcC3KEWUK",
};

const PICOSOL_MINT = "picobAEvs6w7QEknPce34wAE4gknZA9v5tTonnmHYdX";
const MSOL_MINT = "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So";

async function fetchAccountMap(
  rpc: Rpc<SolanaRpcApi>,
  accounts: string[]
): Promise<AccountMap> {
  const map = new Map<string, Account>();
  await Promise.all(
    accounts.map(async (account) => {
      const accountInfo = await rpc
        .getAccountInfo(account as Address, {
          encoding: "base64",
        })
        .send();
      const acc = accountInfo.value!;
      map.set(account, {
        data: new Uint8Array(getBase64Encoder().encode(acc.data[0])),
        owner: acc.owner,
      });
    })
  );
  return map;
}

const rpc = createSolanaRpc("https://api.mainnet-beta.solana.com");

// init
const sanctumRouter = newSanctumRouter();
init(sanctumRouter, [
  {
    mint: PICOSOL_MINT,
    init: PICOSOL_INIT_DATA,
  },
  {
    mint: MSOL_MINT,
  },
]);

// update
const swapMints = [
  {
    swap: "prefundSwapViaStake",
    inp: PICOSOL_MINT,
    out: MSOL_MINT,
  }
];
const accs = accountsToUpdate(sanctumRouter, swapMints);
const accountsToUpdateMap = await fetchAccountMap(rpc, accs);
update(sanctumRouter, swapMints, accountsToUpdateMap);

// quote
const amt = 1_000_000_000n;
const {
  prefundFee,
  quote: {
    routerFee,
    quote: {
      inp,
      out,
      inpFee,
      outFee,
      bridge,
    },
  },
} = quotePrefundSwapViaStake(sanctumRouter, {
  amt,
  inp: PICOSOL_MINT,
  out: MSOL_MINT,
});

// create transaction instruction

// user-provided pubkeys
const signer = ...;
const inpTokenAcc = ...;
const outTokenAcc = ...;

// For PrefundSwapViaStakes and PrefundWithdrawStakes,
// the user must find a u32 bridge stake seed
// that is unused (the bridge stake PDA it creates using
// `findBridgeStakeAccPda()` does not exist as an account onchain)
const bridgeStakedSeed = ...;

const ixUncasted = prefundSwapViaStakeIx(sanctumRouter, {
  amt,
  inp: PICOSOL_MINT,
  out: MSOL_MINT,
  signerInp: inpTokenAcc,
  signerOut: outTokenAcc,
  signer,
  bridgeStakeSeed,

  // optional, this fn runs faster if provided with the
  // bridge stake account's delegated voter that was found beforehand
  bridgeVote: bridge.vote,
});
// return type is compatible with kit,
// but needs to be casted explicitly
const ix = ixUncasted as unknown as IInstruction;
```

## Cloudflare Workers

In Cloudflare Workers and other restricted environments, the default export async init function fails without any args due to path issues of the wasm file, while `initSyncEmbed()` fails due to security restrictions disallowing generation of untrusted wasm code at runtime. The workaround is to copy out the `.wasm` file included in this package into somewhere accessible by these restricted environments, and import it as a module.

```ts
import { initSync } from "@sanctumso/sanctum-router-sdk";
import wasm from "../libs/sanctum_router_sdk_index_bg.wasm";

initSync({ module: wasm });
// or use the package's default export async init function
// instead of `initSync()`
```

## Build

### Prerequisites

- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/)
- `make`
