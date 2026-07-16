import { relaunch } from "@tauri-apps/api/process";
import { checkUpdate, installUpdate } from "@tauri-apps/api/updater";

export type UpdateStatus =
  | "Checking"
  | "Installing"
  | "Restarting"
  | "UpToDate";

const INSTALL_DELAY_MS = 5_000;

export async function runUpdater(
  setStatus: (status: UpdateStatus) => void,
): Promise<void> {
  try {
    const update = await checkUpdate();
    if (!update.shouldUpdate) {
      setStatus("UpToDate");
      return;
    }

    setStatus("Installing");
    await new Promise((resolve) => setTimeout(resolve, INSTALL_DELAY_MS));
    await installUpdate();

    setStatus("Restarting");
    await relaunch();
  } catch (error) {
    console.error("Update check or installation failed", error);
    setStatus("UpToDate");
  }
}
