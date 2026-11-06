<script lang="ts">
  import {
    MULTI_PROVIDERS,
    PLAYER_CLICK_ACTIONS,
    updateConfig,
    type Config,
  } from "$lib/config";
  import { fade } from "svelte/transition";
  import type { ChampSelect } from "$lib/champ_select";
  import ChampSelectPanel from "./champ-select-panel.svelte";
  import LiveGamePanel from "./live-game-panel.svelte";
  import type { LiveGame } from "$lib/live_game";
  import SettingsPanel from "./settings-panel.svelte";
  import IdleOverview from "./idle-overview.svelte";
  import RequeueCountdown from "./requeue-countdown.svelte";
  import { logFrontendError } from "$lib/logging";
  import { isTauriRuntime } from "$lib/runtime";
  import type { LobbyHistoryEntry } from "$lib/lobby_history";
  import type { MatchupBoard } from "$lib/matchups";

  export let config: Config | null = null;
  export let state = "Unknown";
  export let champSelect: ChampSelect | null = null;
  export let matchups: MatchupBoard | null = null;
  export let connected = false;
  export let queuePaused = false;
  export let history: LobbyHistoryEntry[] = [];
  export let activeLobbyId: string | null = null;
  export let liveGame: LiveGame | null = null;
  export let queueName: string | null = null;
  export let requeue: { deadline: number; after: "game" | "queue" | "dodge" } | null = null;
  export let onConfigChange: (config: Config) => void = () => {};
  export let onOpenPage: (page: "history" | "auto") => void = () => {};

  async function handleConfigChange(nextConfig: Config) {
    const previousConfig = config;
    onConfigChange(nextConfig);
    // The browser preview has no backend to save to.
    if (!isTauriRuntime()) return;

    try {
      await updateConfig(nextConfig);
    } catch (error) {
      logFrontendError("Failed to save config", error);
      if (previousConfig) onConfigChange(previousConfig);
    }
  }

  // Champ Select and the live scoreboard need the height, so settings collapse to a one-line
  // summary there (changes made mid-select only apply next game anyway).
  let settingsExpanded = false;
  // The League Client often closes during games, so don't wait for it to say "InProgress".
  $: showingLiveGame = liveGame !== null && state !== "ChampSelect";
  $: inChampSelect = state === "ChampSelect" || showingLiveGame;
  $: if (!inChampSelect) settingsExpanded = false;
  $: settingsSummary = config
    ? [
        MULTI_PROVIDERS.find(({ value }) => value === config?.multiProvider)?.label ?? "OP.GG",
        PLAYER_CLICK_ACTIONS.find(({ value }) => value === config?.playerClickAction)?.label ??
          "Show details",
      ].join(" · ")
    : "";

</script>

<div class="flex h-full min-h-0 flex-col gap-2">
  <!-- Dropped once you leave the post-game screen, or queue after a dodge, yourself. -->
  {#if requeue && (requeue.after === "queue" || (requeue.after === "game" ? state === "EndOfGame" : queuePaused))}
    {#key requeue.deadline}
      <RequeueCountdown deadline={requeue.deadline} after={requeue.after} />
    {/key}
  {/if}
  {#if inChampSelect}
    <button
      type="button"
      class="reveal-panel flex h-9 shrink-0 items-center gap-2 px-3 text-left text-xs transition hover:bg-white/[0.04]"
      aria-expanded={settingsExpanded}
      on:click={() => (settingsExpanded = !settingsExpanded)}
    >
      <span class="font-medium">Settings</span>
      <span class="min-w-0 flex-1 truncate text-muted-foreground">{settingsSummary}</span>
      <svg
        viewBox="0 0 16 16"
        class="h-3.5 w-3.5 shrink-0 text-muted-foreground transition"
        class:rotate-180={settingsExpanded}
        aria-hidden="true"
      >
        <path
          d="m4 6 4 4 4-4"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="1.5"
        />
      </svg>
    </button>
  {/if}
  {#if !inChampSelect || settingsExpanded}
    <SettingsPanel {config} onChange={handleConfigChange} />
  {/if}

  {#if state === "ChampSelect"}
    <div in:fade class="flex min-h-0 flex-1 flex-col">
      <ChampSelectPanel
        {champSelect}
        {matchups}
        countersProvider={config?.countersProvider ?? "lolalytics"}
        {history}
        {activeLobbyId}
        clickAction={config?.playerClickAction ?? "details"}
        provider={config?.multiProvider ?? "opgg"}
        {queueName}
      />
    </div>
  {:else if showingLiveGame && liveGame}
    <div in:fade class="flex min-h-0 flex-1 flex-col">
      <LiveGamePanel game={liveGame} {history} provider={config?.multiProvider ?? "opgg"} />
    </div>
  {:else}
    <IdleOverview {history} {config} {connected} {onOpenPage} />
  {/if}
</div>
