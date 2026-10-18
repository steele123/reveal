<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { logFrontendError, logFrontendInfo } from "$lib/logging";

  export let version = "";
  export let onRemoved: () => void = () => {};

  let status: "idle" | "removing" | "failed" = "idle";

  async function removeOldVersion() {
    status = "removing";
    try {
      const removed = await invoke<boolean>("remove_legacy_install");
      if (removed) {
        logFrontendInfo("Old MSI install removed");
        onRemoved();
        return;
      }
      status = "failed";
    } catch (error) {
      logFrontendError("Failed to remove old MSI install", error);
      status = "failed";
    }
  }
</script>

<div
  class="mx-5 mt-3 flex items-center gap-3 rounded-lg border border-amber-400/20 bg-amber-500/10 px-3 py-2"
>
  <div class="min-w-0 flex-1">
    <div class="text-xs font-medium text-amber-200">
      Finish updating Reveal
    </div>
    <div class="mt-0.5 text-[10px] text-amber-200/70">
      {#if status === "failed"}
        Removal was cancelled or failed. You can try again any time.
      {:else}
        An old copy{version ? ` (v${version})` : ""} is still in Program Files. Removing
        it needs one admin prompt; updates won't after that.
      {/if}
    </div>
  </div>
  <button
    class="shrink-0 rounded-md border border-amber-400/30 bg-amber-500/15 px-2.5 py-1.5 text-[10px] font-medium text-amber-100 transition hover:bg-amber-500/25 disabled:opacity-60"
    disabled={status === "removing"}
    on:click={removeOldVersion}
  >
    {status === "removing" ? "Removing…" : "Remove old version"}
  </button>
</div>
