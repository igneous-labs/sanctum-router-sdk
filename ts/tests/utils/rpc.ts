import type { OwnedAccount } from "@sanctumso/sanctum-router";
import {
  getBase64Encoder,
  type Address,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";

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
      map.set(account, {
        data: new Uint8Array(
          getBase64Encoder().encode(accountInfo.value!.data[0])
        ),
        owner: accountInfo.value!.owner,
        lamports: accountInfo.value!.lamports,
      });
    })
  );
  return map;
}
