<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import type { ChampSelect } from "$lib/champ_select";
  import { Button } from "./ui/button";

  export let champSelect: ChampSelect | null = null;

  function openMultiLink() {
    void invoke<void>("open_opgg_link").catch((error) => {
      console.error("Failed to open multi link", error);
    });
  }
</script>

<div class="flex flex-col gap-5 w-full">
  <div class="grid grid-cols-2 items-start gap-y-1 gap-x-2 text-sm">
    {#if champSelect}
      {#each champSelect.participants as participant (participant.puuid)}
        <div
          class="flex flex-col items-center justify-center border bg-primary-foreground rounded-md text-xs h-9"
        >
          <div class="line-clamp-1">
            {participant.game_name}#{participant.game_tag}
          </div>
        </div>
      {/each}
    {:else}
      {#each Array.from({ length: 5 }) as _, index (index)}
        <div
          class="bg-primary-foreground border animate-pulse h-9 w-full rounded-md"
        />
      {/each}
    {/if}
  </div>

  <Button
    class="h-9 absolute right-4 w-[180px] bottom-[52px]"
    size="sm"
    disabled={!champSelect}
    on:click={openMultiLink}
  >
    Open Multi Link
  </Button>
</div>
