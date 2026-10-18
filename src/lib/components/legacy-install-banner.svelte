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
      Remove the old copy of Reveal
    </div>
    <div class="mt-0.5 text-[10px] leading-snug text-amber-200/70">
      {#if status === "removing"}
        Choose <span class="font-semibold text-amber-100">Yes</span> on the Windows
        prompt to finish.
      {:else if status === "failed"}
        The old copy wasn't removed. Click Try again and choose
        <span class="font-semibold text-amber-100">Yes</span> on the Windows prompt.
      {:else}
        Reveal no longer needs admin to update. Click Remove, then choose
        <span class="font-semibold text-amber-100">Yes</span> on the Windows prompt to
        delete the old{version ? ` v${version}` : ""} copy. Your settings and history stay.
      {/if}
    </div>
  </div>
  <button
    class="shrink-0 rounded-md border border-amber-400/30 bg-amber-500/15 px-2.5 py-1.5 text-[10px] font-medium text-amber-100 transition hover:bg-amber-500/25 disabled:opacity-60"
    disabled={status === "removing"}
    on:click={removeOldVersion}
  >
    {status === "removing" ? "Waiting…" : status === "failed" ? "Try again" : "Remove"}
  </button>
</div>
