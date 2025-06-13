import type { OwnedAccount } from "@sanctumso/sanctum-router";
import {
  getBase64Encoder,
  type Address,
  type Rpc,
  type SolanaRpcApi,
  createSolanaRpc,
} from "@solana/kit";

export function localRpc(): Rpc<SolanaRpcApi> {
  return createSolanaRpc("http://localhost:8899");
}

export async function fetchAccountMap(
  rpc: Rpc<SolanaRpcApi>,
  accounts: string[]
): Promise<Map<string, OwnedAccount>> {
  let map = new Map<string, OwnedAccount>();
  await Promise.all(
    accounts.map(async (account) => {
      const accountInfo = await rpc
        .getAccountInfo(account as Address, {
          encoding: "base64",
        })
        .send();
      const a = accountInfo.value;
      if (a == null) {
        throw new Error(`Missing account ${account}`);
      }
      map.set(account, {
        data: new Uint8Array(getBase64Encoder().encode(a.data[0])),
        owner: a.owner,
        lamports: a.lamports,
      });
    })
  );
  return map;
}
