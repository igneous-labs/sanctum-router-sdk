import type { OwnedAccount } from "@sanctumso/sanctum-router";
import {
  address,
  createKeyPairSignerFromBytes,
  getBase64Encoder,
  type Address,
  type KeyPairSigner,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";
import { readFileSync } from "fs";

export function readTestFixturesJsonFile(fname: string): any {
  return JSON.parse(
    readFileSync(
      `${import.meta.dirname}/../../../test-fixtures/${fname}.json`,
      "utf8"
    )
  );
}

export function readTestFixturesAccPk(fname: string): Address<string> {
  const { pubkey } = readTestFixturesJsonFile(fname);
  return address(pubkey);
}

export function readTestFixturesKeypair(
  fname: string
): Promise<KeyPairSigner<string>> {
  const bytes = JSON.parse(
    readFileSync(
      `${import.meta.dirname}/../../../test-fixtures/key/${fname}.json`,
      "utf8"
    )
  );
  return createKeyPairSignerFromBytes(new Uint8Array(bytes));
}

export const NATIVE_MINT = address(
  "So11111111111111111111111111111111111111112"
);

export const MSOL_MINT = address("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So");

export function mapTup<T extends readonly any[], U>(
  tuple: T,
  callback: (value: T[number], index: number) => U
): { [K in keyof T]: U } {
  return tuple.map(callback) as any;
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
