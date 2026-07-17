<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import type { ChampSelect } from "$lib/champ_select";
  import { Button } from "./ui/button";
  import { logFrontendError } from "$lib/logging";

  export let champSelect: ChampSelect | null = null;

  const POSITION_LABELS: Record<string, string> = {
    top: "Top",
    jungle: "Jungle",
    middle: "Mid",
    bottom: "ADC",
    utility: "Support",
  };

  function positionLabel(position: string): string {
    return (
      POSITION_LABELS[position.toLowerCase()] ||
      `${position.slice(0, 1).toUpperCase()}${position.slice(1)}`
    );
  }

  function openMultiLink() {
    void invoke<void>("open_opgg_link").catch((error) => {
      logFrontendError("Failed to open multi link", error);
    });
  }
</script>

<section class="flex min-h-0 flex-1 flex-col gap-3">
  <div class="flex items-center justify-between">
    <div>
      <div class="text-sm font-semibold">Champ Select</div>
      <div class="text-[11px] text-muted-foreground">
        Teammate identities are ready.
      </div>
    </div>
    <div
      class="rounded-full border border-blue-400/20 bg-blue-500/10 px-2 py-1 text-[10px] font-medium text-blue-300"
    >
      {champSelect?.participants.length ?? 0}/5 found
    </div>
  </div>

  <div class="grid grid-cols-2 gap-2 text-sm">
    {#if champSelect}
      {#each champSelect.participants as participant, index (participant.puuid)}
        <div
          class:col-span-2={champSelect.participants.length % 2 === 1 &&
            index === champSelect.participants.length - 1}
          class="reveal-panel flex h-11 min-w-0 items-center gap-2.5 px-3"
        >
          <div
            class="grid h-6 min-w-7 shrink-0 place-items-center rounded-md bg-blue-500/10 px-1 text-[9px] font-semibold text-blue-300"
            title={participant.pick_turn === undefined
              ? `Player ${index + 1}`
              : `Pick ${participant.pick_turn + 1}`}
          >
            {participant.pick_turn === undefined
              ? String(index + 1).padStart(2, "0")
              : `P${participant.pick_turn + 1}`}
          </div>
          <div class="min-w-0 flex-1 truncate text-xs font-medium">
            {participant.game_name}<span class="text-muted-foreground"
              >#{participant.game_tag}</span
            >
          </div>
          {#if participant.assigned_position}
            <div
              class="shrink-0 rounded-md border border-white/[0.07] bg-white/[0.035] px-1.5 py-0.5 text-[9px] font-medium text-muted-foreground"
            >
              {positionLabel(participant.assigned_position)}
            </div>
          {/if}
        </div>
      {/each}
    {:else}
      {#each Array.from({ length: 5 }) as _, index (index)}
        <div
          class:col-span-2={index === 4}
          class="reveal-panel flex h-11 animate-pulse items-center gap-2.5 px-3"
        >
          <div class="h-6 w-6 rounded-md bg-white/5" />
          <div class="h-2.5 w-28 rounded-full bg-white/5" />
        </div>
      {/each}
    {/if}
  </div>

  <Button
    class="mt-auto h-10 w-full bg-blue-500 text-sm shadow-[0_8px_24px_rgba(59,130,246,0.18)] hover:bg-blue-400"
    disabled={!champSelect}
    on:click={openMultiLink}
  >
    Open team multi-search
    <svg
      viewBox="0 0 16 16"
      class="ml-2 h-3.5 w-3.5"
      aria-hidden="true"
    >
      <path
        d="M6 3h7v7m0-7L6.5 9.5M11 9v3a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1h3"
        fill="none"
        stroke="currentColor"
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-width="1.25"
      />
    </svg>
  </Button>
</section>
