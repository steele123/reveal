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

  function formatState(value: string): string {
    if (value === "InProgress") return "In game";
    if (value === "ChampSelect") return "Champ Select";
    if (value === "Unknown") return "Not connected";
    return value.replace(/([a-z])([A-Z])/g, "$1 $2");
  }
</script>

<div class="flex h-full min-h-0 flex-col gap-3">
  <SettingsPanel {config} onChange={handleConfigChange} />

  {#if state === "ChampSelect"}
    <div in:fade class="flex min-h-0 flex-1 flex-col">
      <ChampSelectPanel {champSelect} />
    </div>
  {:else}
    <div class="grid grid-cols-2 gap-3">
      <div class="reveal-panel px-3.5 py-3">
        <div
          class="text-[10px] font-medium uppercase tracking-[0.12em] text-muted-foreground"
        >
          Client state
        </div>
        <div class="mt-1.5 flex items-center gap-2 text-sm font-semibold">
          <span
            class="h-1.5 w-1.5 rounded-full"
            class:bg-blue-400={connected}
            class:bg-amber-400={!connected}
          />
          {formatState(state)}
        </div>
      </div>
      <div class="reveal-panel px-3.5 py-3">
        <div
          class="text-[10px] font-medium uppercase tracking-[0.12em] text-muted-foreground"
        >
          Revealed lobbies
        </div>
        <div class="mt-1.5 text-sm font-semibold tabular-nums">
          <RevealCount />
        </div>
      </div>
    </div>

    <div
      in:fade
      class="reveal-panel flex min-h-0 flex-1 items-center px-5 py-4"
    >
      <div class="flex items-center gap-4">
        <div
          class={`grid h-10 w-10 shrink-0 place-items-center rounded-xl border ${
            connected
              ? "border-blue-400/20 bg-blue-500/10 text-blue-300"
              : "border-amber-400/20 bg-amber-500/10 text-amber-300"
          }`}
        >
          {#if state === "InProgress"}
            <svg viewBox="0 0 20 20" class="h-5 w-5" aria-hidden="true">
              <path d="M7 5.5 13 10l-6 4.5v-9Z" fill="currentColor" />
            </svg>
          {:else}
            <span class="relative flex h-3 w-3">
              <span
                class="absolute h-full w-full animate-ping rounded-full bg-current opacity-30"
              />
              <span class="relative h-3 w-3 rounded-full bg-current" />
            </span>
          {/if}
        </div>
        <div>
          <div class="text-sm font-semibold">
            {#if state === "InProgress"}
              Game in progress
            {:else if !connected}
              Looking for League
            {:else}
              Ready for Champ Select
            {/if}
          </div>
          <div class="mt-1 max-w-[360px] text-xs leading-relaxed text-muted-foreground">
            {#if state === "InProgress"}
              Reveal is standing by and will be ready for your next lobby.
            {:else if !connected}
              Start League, or restart it and run Reveal as administrator if it is already open.
            {:else}
              Join a lobby and queue normally. Teammate names will appear here automatically.
            {/if}
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
