<script lang="ts">
  import { onMount } from "svelte";
  import { fetch } from "@tauri-apps/api/http";
  import { IS_DEV } from "../lib/constants";

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
    const resp = await fetch("https://hyperboost.gg/api/reveal/stats");

    const data = resp.data as {
      version: string;
      revealedLobbies: number;
      downloads: number;
    };

    revealCount = data.revealedLobbies;
  }
</script>

{revealCount}
