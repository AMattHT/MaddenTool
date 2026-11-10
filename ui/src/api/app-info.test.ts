import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { getAppInfo } from "./app-info";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const valid = {
  appName: "MaddenTool",
  appVersion: "1.2.3",
  game: "Madden 27",
  platform: "windows",
  arch: "x86_64",
};

describe("getAppInfo", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("invokes the Rust command and returns its data", async () => {
    vi.mocked(invoke).mockResolvedValue(valid);
    await expect(getAppInfo()).resolves.toEqual(valid);
    expect(invoke).toHaveBeenCalledWith("get_app_info");
  });

  it("rejects when the native call fails instead of inventing data", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("ipc down"));
    await expect(getAppInfo()).rejects.toThrow("ipc down");
  });

  it.each([
    ["a null result", null],
    ["a non-object result", "MaddenTool"],
    ["a missing field", { ...valid, arch: undefined }],
    ["a wrong field type", { ...valid, appVersion: 1 }],
    ["an empty field", { ...valid, platform: "" }],
    ["snake_case fields", { app_name: "MaddenTool", app_version: "1.2.3" }],
  ])("rejects a mismatched contract: %s", async (_name, payload) => {
    vi.mocked(invoke).mockResolvedValue(payload);
    await expect(getAppInfo()).rejects.toThrow(/unexpected app information/i);
  });
});
