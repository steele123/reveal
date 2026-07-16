<script lang="ts">
  import { updateConfig, type Config } from "$lib/config";
  import { fade } from "svelte/transition";
  import RevealCount from "./reveal-count.svelte";
  import type { ChampSelect } from "$lib/champ_select";
  import ChampSelectPanel from "./champ-select-panel.svelte";
  import SettingsPanel from "./settings-panel.svelte";

  export let config: Config | null = null;
  export let state = "Unknown";
  export let champSelect: ChampSelect | null = null;
  export let connected = false;
  export let onConfigChange: (config: Config) => void = () => {};

  async function handleConfigChange(nextConfig: Config) {
    const previousConfig = config;
    onConfigChange(nextConfig);

    try {
      await updateConfig(nextConfig);
    } catch (error) {
      console.error("Failed to save config", error);
      if (previousConfig) onConfigChange(previousConfig);
    }
  }
</script>

<div class="flex flex-col gap-2">
  <SettingsPanel {config} onChange={handleConfigChange} />
  <div class="grid grid-cols-2 text-sm">
    <div class="flex flex-col">
      <div class="text-muted-foreground text-xs">State</div>
      <div>{state}</div>
    </div>
    <div class="flex flex-col">
      <div class="text-muted-foreground text-xs">Revealed Champ Selects</div>
      <div>
        <RevealCount />
      </div>
    </div>
  </div>
  {#if state === "ChampSelect"}
    <div in:fade>
      <ChampSelectPanel {champSelect} />
    </div>
  {:else if state === "InProgress"}
    <div in:fade class="flex gap-2 items-center animate-pulse">In Game</div>
  {:else if !connected}
    <div in:fade class="flex gap-2 items-center animate-pulse">
      Trying to find League Client...
    </div>
    <div
      class="text-xs p-2 rounded bg-accent border flex gap-2 text-muted-foreground"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="20"
        height="20"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="lucide lucide-info"
        ><circle cx="12" cy="12" r="10" /><path d="M12 16v-4" /><path
          d="M12 8h.01"
        /></svg
      >
      Issues Connecting? <br /> Try restarting the League Client and running Reveal
      as Administrator.
    </div>
  {:else}
    <div in:fade class="flex gap-2 items-center animate-pulse">
      Waiting for Champ Select...
    </div>
  {/if}
</div>
