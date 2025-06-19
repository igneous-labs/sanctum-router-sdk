import {
  init,
  isInit,
  newSanctumRouter,
  initSyncEmbed,
} from "@sanctumso/sanctum-router";
import { beforeAll, describe, expect, it } from "vitest";
import {
  BSOL_MINT,
  MSOL_MINT,
  NATIVE_MINT,
  PICOSOL_INIT_DATA,
  PICOSOL_MINT,
  STSOL_MINT,
} from "../utils";

describe("Init Test", () => {
  beforeAll(() => {
    initSyncEmbed();
  });

  it("Non-SPLs do not require init()", () => {
    const router = newSanctumRouter();
    const mints = [NATIVE_MINT, STSOL_MINT, MSOL_MINT];
    const isInitRet = isInit(router, mints);
    expect(isInitRet).toStrictEqual(
      new Uint8Array(Array.from({ length: mints.length }, () => 1))
    );
  });

  it("isInit returns false for uninit SPL", () => {
    const router = newSanctumRouter();
    const mints = [PICOSOL_MINT, BSOL_MINT];
    const isInitRet = isInit(router, mints);
    expect(isInitRet).toStrictEqual(
      new Uint8Array(Array.from({ length: mints.length }, () => 0))
    );
  });

  it("isInit returns true for init SPL", () => {
    const router = newSanctumRouter();
    const mints = [PICOSOL_MINT, BSOL_MINT];
    init(router, [
      { mint: PICOSOL_MINT, init: { pool: "spl", ...PICOSOL_INIT_DATA } },
    ]);
    const isInitRet = isInit(router, mints);
    expect(isInitRet[0]).toEqual(1);
    expect(isInitRet[1]).toEqual(0);
  });
});
