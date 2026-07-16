<script lang="ts">
  import { onMount } from "svelte";
  import { fetch } from "@tauri-apps/api/http";

  import { isTauriRuntime } from "$lib/runtime";

  interface RevealStats {
    version: string;
    revealedLobbies: number;
    downloads: number;
  }

  let revealCount: number | null = null;

  onMount(() => {
    updateRevealCount();

    const clear = setInterval(async () => {
      await updateRevealCount();
    }, 60000);

    return () => {
      clearInterval(clear);
    };
  });

  async function updateRevealCount() {
    try {
      let data: RevealStats;
      if (isTauriRuntime()) {
        const response = await fetch<RevealStats>(
          "https://lobbyreveal.app/api/reveal/stats",
        );
        if (!response.ok) throw new Error("Failed to fetch reveal stats");
        data = response.data;
      } else {
        const response = await window.fetch(
          "https://lobbyreveal.app/api/reveal/stats",
        );
        if (!response.ok) throw new Error("Failed to fetch reveal stats");
        data = (await response.json()) as RevealStats;
      }

      revealCount = data.revealedLobbies;
    } catch (error) {
      console.error("Failed to fetch reveal stats", error);
      revealCount = null;
    }
  }
</script>

{#if revealCount === null}
  <span class="text-muted-foreground" title="Reveal count unavailable">—</span>
{:else}
  {revealCount.toLocaleString()}
{/if}
