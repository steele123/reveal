<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { fade, scale } from "svelte/transition";
  import { championIconUrl, formatAssignedPosition } from "$lib/champ_select";
  import { logFrontendError } from "$lib/logging";
  import { isTauriRuntime } from "$lib/runtime";
  import {
    csLead,
    formatLead,
    formatWinRate,
    leadClass,
    MIN_RELIABLE_GAMES,
    type Counter,
    type EnemyMatchups,
  } from "$lib/matchups";

  /** The counter you clicked, and the enemy it counters. */
  export let counter: Counter;
  export let enemy: EnemyMatchups;
  /** The site the numbers came from, for the matchup page button. */
  export let providerLabel = "Lolalytics";
  /** Opens the matchup's page; null when there isn't one. */
  export let onOpenMatchup: (() => void) | null = null;
  export let onClose: () => void = () => {};

  let hovering = false;
  let refusal: string | null = null;

  $: stats = counter.stats;
  $: subtitle = [
    `vs ${enemy.champion.name}`,
    enemy.role && formatAssignedPosition(enemy.role) + (enemy.roleGuessed ? " (likely)" : ""),
  ]
    .filter(Boolean)
    .join(" · ");
  $: tiles = [
    {
      label: "Win rate",
      value: formatWinRate(stats.winRate),
      tone: leadClass(stats.winRateDelta ?? stats.winRate - 50, 1),
    },
    {
      label: "Gold @15",
      value: formatLead(stats.goldDiff15),
      tone: leadClass(stats.goldDiff15, 50),
    },
    { label: "CS @15", value: formatLead(csLead(stats)), tone: leadClass(csLead(stats), 1) },
    {
      label: "Games",
      value: stats.games.toLocaleString("en-US"),
      tone: "text-foreground",
    },
  ];

  /** Shows the champion as your pick in League, without locking it in. */
  async function hover() {
    if (hovering) return;
    if (!isTauriRuntime()) {
      onClose();
      return;
    }
    hovering = true;
    refusal = null;
    try {
      await invoke("hover_champion", { championId: counter.champion.id });
      onClose();
    } catch (error) {
      refusal = typeof error === "string" ? error : "Reveal couldn't hover it.";
      logFrontendError("Failed to hover champion", error);
    } finally {
      hovering = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") onClose();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div
  class="fixed inset-0 z-50 grid place-items-center bg-black/60 p-6 backdrop-blur-sm"
  transition:fade={{ duration: 120 }}
  on:click|self={onClose}
>
  <div
    role="dialog"
    aria-modal="true"
    aria-label={`${counter.champion.name} vs ${enemy.champion.name}`}
    class="relative w-full max-w-sm rounded-xl border border-white/10 bg-[#0b1220] p-4 shadow-2xl"
    transition:scale={{ duration: 120, start: 0.97 }}
  >
    <button
      class="absolute right-3 top-3 grid h-7 w-7 place-items-center rounded-md text-muted-foreground transition hover:bg-white/5 hover:text-foreground"
      aria-label="Close"
      on:click={onClose}
    >
      <svg viewBox="0 0 16 16" class="h-3.5 w-3.5" aria-hidden="true">
        <path
          d="m4 4 8 8M12 4l-8 8"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-width="1.5"
        />
      </svg>
    </button>

    <div class="flex items-center gap-3 pr-8">
      <img
        src={championIconUrl(counter.champion.id)}
        alt=""
        class="h-12 w-12 shrink-0 rounded-lg bg-white/5 object-cover"
      />
      <div class="min-w-0">
        <div class="truncate text-base font-semibold">{counter.champion.name}</div>
        <div class="mt-0.5 truncate text-xs text-muted-foreground">
          {subtitle}
        </div>
      </div>
    </div>

    <div class="mt-4 grid grid-cols-4 gap-1.5">
      {#each tiles as tile (tile.label)}
        <div class="reveal-panel px-2 py-1.5">
          <div class="text-[9px] font-medium uppercase tracking-[0.08em] text-muted-foreground">
            {tile.label}
          </div>
          <div class="mt-0.5 text-sm font-semibold tabular-nums {tile.tone}">{tile.value}</div>
        </div>
      {/each}
    </div>
    {#if stats.games < MIN_RELIABLE_GAMES}
      <div class="mt-2 text-[11px] text-muted-foreground">
        Small sample, so take it with a grain of salt.
      </div>
    {/if}

    {#if refusal}
      <div class="mt-3 rounded-md bg-red-500/10 px-3 py-2 text-[11px] text-red-200" role="alert">
        {refusal}
      </div>
    {/if}

    <div class="mt-4 flex gap-2">
      {#if onOpenMatchup}
        <button
          class="h-9 shrink-0 rounded-md border border-white/10 bg-white/[0.035] px-3 text-xs font-medium transition hover:bg-white/[0.07]"
          title={`${counter.champion.name} vs ${enemy.champion.name} on ${providerLabel}`}
          on:click={onOpenMatchup}
        >
          {providerLabel} ↗
        </button>
      {/if}
      <button
        class="h-9 flex-1 rounded-md bg-blue-500 text-sm font-medium text-white shadow-[0_8px_24px_rgba(59,130,246,0.18)] transition hover:bg-blue-400 disabled:opacity-60"
        disabled={hovering}
        title="Shows {counter.champion.name} as your pick in League. It doesn't lock it in."
        on:click={hover}
      >
        {hovering ? "Hovering…" : `Hover ${counter.champion.name}`}
      </button>
    </div>
  </div>
</div>
