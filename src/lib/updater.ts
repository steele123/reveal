import { relaunch } from "@tauri-apps/api/process";
import { checkUpdate, installUpdate } from "@tauri-apps/api/updater";
import { logFrontendError, logFrontendInfo } from "$lib/logging";

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
    logFrontendInfo("Update check started");
    const update = await checkUpdate();
    if (!update.shouldUpdate) {
      logFrontendInfo("Reveal is up to date");
      setStatus("UpToDate");
      return;
    }

    setStatus("Installing");
    logFrontendInfo("Installing Reveal update");
    await new Promise((resolve) => setTimeout(resolve, INSTALL_DELAY_MS));
    await installUpdate();

    setStatus("Restarting");
    logFrontendInfo("Update installed; restarting Reveal");
    await relaunch();
  } catch (error) {
    logFrontendError("Update check or installation failed", error);
    setStatus("UpToDate");
  }
}
