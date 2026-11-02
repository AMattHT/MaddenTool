import { invoke } from "@tauri-apps/api/core";

export type AppInfo = {
  appName: string;
  appVersion: string;
  game: string;
  platform: string;
  arch: string;
};

const FIELDS = ["appName", "appVersion", "game", "platform", "arch"] as const;

function isAppInfo(value: unknown): value is AppInfo {
  if (typeof value !== "object" || value === null) return false;
  const record = value as Record<string, unknown>;
  return FIELDS.every((field) => {
    const entry = record[field];
    return typeof entry === "string" && entry.length > 0;
  });
}

/** Reads application facts from Rust. Rejects on IPC failure or a contract mismatch. */
export async function getAppInfo(): Promise<AppInfo> {
  const value = await invoke<unknown>("get_app_info");
  if (!isAppInfo(value)) {
    throw new Error("The backend returned unexpected app information.");
  }
  return value;
}
