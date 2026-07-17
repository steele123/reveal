import { invoke } from "@tauri-apps/api/tauri";
import { isTauriRuntime } from "$lib/runtime";

type LogLevel = "info" | "warn" | "error";

function describeError(error: unknown): string {
  if (error instanceof Error) return error.stack || error.message;
  if (typeof error === "string") return error;

  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}

function writeLog(level: LogLevel, message: string) {
  if (!isTauriRuntime()) return;
  void invoke<void>("write_frontend_log", { level, message }).catch(() => {});
}

export function logFrontendInfo(message: string) {
  console.info(message);
  writeLog("info", message);
}

export function logFrontendWarning(message: string, error?: unknown) {
  const detail = error === undefined ? message : `${message}: ${describeError(error)}`;
  console.warn(message, error ?? "");
  writeLog("warn", detail);
}

export function logFrontendError(message: string, error?: unknown) {
  const detail = error === undefined ? message : `${message}: ${describeError(error)}`;
  console.error(message, error ?? "");
  writeLog("error", detail);
}
