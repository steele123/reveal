<script lang="ts">
  import { onDestroy } from "svelte";
  import type { MultiProvider } from "$lib/config";
  import type { LobbyHistoryEntry } from "$lib/lobby_history";
  import {
    copyLink,
    createMultiSearchLink,
    createOpggProfileLink,
    openExternalLink,
  } from "$lib/link_actions";
  import { logFrontendError } from "$lib/logging";
  import { formatAssignedPosition } from "$lib/champ_select";

  export let history: LobbyHistoryEntry[] = [];
  export let provider: MultiProvider = "opgg";
  export let onClear: () => void = () => {};

  let message = "";
  let messageTimer: ReturnType<typeof setTimeout> | undefined;
  let confirmingClear = false;

  const dateFormatter = new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });

  $: providerName = {
    opgg: "OP.GG",
    deeplol: "DeepLoL",
    ugg: "U.GG",
    tracker: "Tracker.gg",
  }[provider];

  function showMessage(nextMessage: string) {
    message = nextMessage;
    if (messageTimer) clearTimeout(messageTimer);
    messageTimer = setTimeout(() => (message = ""), 2_000);
  }

  function formatDate(value: string): string {
    return dateFormatter.format(new Date(value));
  }

  async function openMulti(entry: LobbyHistoryEntry) {
    try {
      await openExternalLink(createMultiSearchLink(entry.participants, provider));
    } catch (error) {
      logFrontendError("Failed to open historical multi link", error);
      showMessage("Could not open the link");
    }
  }

  async function copyMulti(entry: LobbyHistoryEntry) {
    try {
      await copyLink(createMultiSearchLink(entry.participants, provider));
      showMessage("Multi-search link copied");
    } catch (error) {
      logFrontendError("Failed to copy historical multi link", error);
      showMessage("Could not copy the link");
    }
  }

  async function copyProfile(entry: LobbyHistoryEntry, index: number) {
    try {
      await copyLink(createOpggProfileLink(entry.participants[index]));
      showMessage("OP.GG profile link copied");
    } catch (error) {
      logFrontendError("Failed to copy profile link", error);
      showMessage("Could not copy the link");
    }
  }

  function handleClear() {
    if (!confirmingClear) {
      confirmingClear = true;
      return;
    }

    onClear();
    confirmingClear = false;
    showMessage("Lobby history cleared");
  }

  onDestroy(() => {
    if (messageTimer) clearTimeout(messageTimer);
  });
</script>

<section class="flex h-full min-h-0 flex-col gap-3">
  <div class="flex shrink-0 items-start justify-between gap-4">
    <div>
      <div class="flex items-center gap-2">
        <h1 class="text-sm font-semibold">Lobby history</h1>
        <span
          class="rounded-full border border-white/10 bg-white/5 px-2 py-0.5 text-[10px] tabular-nums text-muted-foreground"
        >
          {history.length}
        </span>
      </div>
      <p class="mt-1 text-[11px] text-muted-foreground">
        Recent teammate reveals stored on this device.
      </p>
    </div>

    {#if history.length > 0}
      <button
        class:text-red-300={confirmingClear}
        class="rounded-md px-2.5 py-1.5 text-[11px] text-muted-foreground transition hover:bg-red-500/10 hover:text-red-300 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-red-400"
        on:click={handleClear}
        on:blur={() => (confirmingClear = false)}
      >
        {confirmingClear ? "Confirm clear" : "Clear history"}
      </button>
    {/if}
  </div>

  {#if message}
    <div
      class="absolute bottom-12 left-1/2 z-10 -translate-x-1/2 rounded-full border border-blue-400/20 bg-slate-950/95 px-3 py-1.5 text-[11px] text-blue-200 shadow-xl"
      role="status"
    >
      {message}
    </div>
  {/if}

  {#if history.length === 0}
    <div class="reveal-panel grid min-h-0 flex-1 place-items-center px-8 text-center">
      <div>
        <div
          class="mx-auto grid h-11 w-11 place-items-center rounded-xl border border-blue-400/20 bg-blue-500/10 text-blue-300"
        >
          <svg viewBox="0 0 20 20" class="h-5 w-5" aria-hidden="true">
            <path
              d="M10 5.5V10l3 1.75M4.5 5.5H2.75V3.75M3.1 7.3A7 7 0 1 1 3 12"
              fill="none"
              stroke="currentColor"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="1.35"
            />
          </svg>
        </div>
        <div class="mt-3 text-sm font-semibold">No lobby reveals yet</div>
        <p class="mt-1 max-w-[300px] text-xs leading-relaxed text-muted-foreground">
          Revealed teammates will appear here automatically when Champ Select starts.
        </p>
      </div>
    </div>
  {:else}
    <div class="reveal-scrollbar min-h-0 flex-1 space-y-3 overflow-y-auto pr-1.5">
      {#each history as entry (entry.id)}
        <article class="reveal-panel p-3">
          <div class="flex items-center gap-2">
            <div class="min-w-0">
              <div class="text-xs font-semibold">{formatDate(entry.revealedAt)}</div>
              <div class="mt-0.5 text-[10px] text-muted-foreground">
                {entry.participants.length}/5 players revealed
              </div>
            </div>

            <div
              class={`ml-auto rounded-full border px-2 py-1 text-[9px] font-medium ${
                entry.gameStartedAt
                  ? "border-emerald-400/20 bg-emerald-500/10 text-emerald-300"
                  : "border-amber-400/20 bg-amber-500/10 text-amber-300"
              }`}
            >
              {entry.gameStartedAt ? "Game started" : "Lobby reveal"}
            </div>

            <button
              class="rounded-md border border-white/10 bg-white/5 px-2.5 py-1.5 text-[10px] font-medium text-foreground transition hover:border-blue-400/30 hover:bg-blue-500/10"
              on:click={() => openMulti(entry)}
            >
              Open {providerName}
            </button>
            <button
              class="grid h-7 w-7 place-items-center rounded-md text-muted-foreground transition hover:bg-white/5 hover:text-blue-300 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
              aria-label={`Copy ${providerName} multi-search link`}
              title={`Copy ${providerName} multi-search link`}
              on:click={() => copyMulti(entry)}
            >
              <svg viewBox="0 0 16 16" class="h-3.5 w-3.5" aria-hidden="true">
                <path
                  d="M5.5 5.5h6v7h-6v-7Zm-2 4v-6h6"
                  fill="none"
                  stroke="currentColor"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="1.25"
                />
              </svg>
            </button>
          </div>

          <div class="mt-2.5 grid grid-cols-2 gap-1.5">
            {#each entry.participants as participant, index (participant.puuid || `${participant.game_name}-${participant.game_tag}`)}
              <div
                class:col-span-2={entry.participants.length % 2 === 1 && index === entry.participants.length - 1}
                class="flex h-8 min-w-0 items-center rounded-lg border border-white/[0.07] bg-white/[0.025] pl-2.5 pr-1"
              >
                <div class="min-w-0 flex-1 truncate text-[11px] font-medium">
                  {participant.game_name}<span class="text-muted-foreground">#{participant.game_tag}</span>
                </div>
                {#if participant.assigned_position}
                  <div
                    class="mr-1 shrink-0 rounded border border-white/[0.07] bg-white/[0.035] px-1.5 py-0.5 text-[8px] font-medium text-muted-foreground"
                  >
                    {formatAssignedPosition(participant.assigned_position)}
                  </div>
                {/if}
                <button
                  class="grid h-6 w-6 shrink-0 place-items-center rounded text-muted-foreground transition hover:bg-white/5 hover:text-blue-300 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                  aria-label={`Copy ${participant.game_name}'s OP.GG profile link`}
                  title="Copy OP.GG profile link"
                  on:click={() => copyProfile(entry, index)}
                >
                  <svg viewBox="0 0 16 16" class="h-3 w-3" aria-hidden="true">
                    <path
                      d="M5.5 5.5h6v7h-6v-7Zm-2 4v-6h6"
                      fill="none"
                      stroke="currentColor"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="1.25"
                    />
                  </svg>
                </button>
              </div>
            {/each}
          </div>
        </article>
      {/each}
    </div>
  {/if}
</section>
