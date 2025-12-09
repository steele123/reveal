<script lang="ts">
  import { onMount } from "svelte";
  import { fetch } from "@tauri-apps/api/http";
  import { IS_DEV } from "$lib/constants";

  let revealCount = 0;

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
      const resp = await fetch("https://lobbyreveal.app/api/reveal/stats");
      if (!resp.ok) {
        throw new Error("Failed to fetch reveal stats");
      }

      const data = resp.data as {
        version: string;
        revealedLobbies: number;
        downloads: number;
      };

      revealCount = data.revealedLobbies;
    } catch (e) {
      console.error(e);

      revealCount = -1;
    }
  }
</script>

{#if revealCount === -1}
  <div class="text-red-500">Failed to fetch analytics</div>
{:else}
  {revealCount}
{/if}
