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
  initAccounts,
  prefundSwapViaStakeIx,
  quotePrefundSwapViaStake,
  update,
  type SplPoolAccounts,
} from "@sanctumso/sanctum-router";

// SPL stake pools (all 3 deploys) must have their stake pool and validator list
// addresses known beforehand and explicitly passed in at initialization time
const SPL_POOL_ACCOUNTS: SplPoolAccounts[] = [
  {
    pool: "8Dv3hNYcEWEaa4qVx9BTN1Wfvtha1z8cWDUXb7KVACVe",
    validatorList: "46A5KjX8J6FAUTXwcE8iJkmM7igK3v8vy1MD74cZNWVE",
  },
  {
    pool: "stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi",
    validatorList: "1istpXjy8BM7Vd5vPfA485frrV7SRJhgq5vs3sskWmc",
  },
];

const BSOL_MINT = "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1";
const PICOSOL_MINT = "picobAEvs6w7QEknPce34wAE4gknZA9v5tTonnmHYdX";

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
const initAccs = initAccounts(spls);
const accounts = await fetchAccountMap(rpc, initAccs);
const sanctumRouter = init(spls, accounts);

// update
const swapMints = [
  {
    prefundSwapViaStake: {
      inp: PICOSOL,
      out: BSOL,
    }
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
    },
  },
} = quotePrefundSwapViaStake(sanctumRouter, {
  amt,
  inp: PICOSOL,
  out: BSOL,
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
  inp: PICOSOL,
  out: BSOL,
  signerInp: inpTokenAcc,
  signerOut: outTokenAcc,
  signer,
  bridgeStakeSeed,
});
// return type is compatible with kit,
// but needs to be casted explicitly
const ix = ixUncasted as unknown as IInstruction;
```

## Build

### Prerequisites

- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/)
- `make` (optional, you can just run the `wasm-pack` commands manually)
