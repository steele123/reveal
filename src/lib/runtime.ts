export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_IPC__" in window;
}
