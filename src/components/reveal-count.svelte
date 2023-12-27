<script lang="ts">
  import { onMount } from "svelte";
  import { IS_DEV } from "../lib/constants";

  let revealCount = 0;

  onMount(async () => {
    const eventSource = new EventSource("https://api.hyperboost.gg/reveal/events");
    eventSource.addEventListener("lobby_reveal", (event) => {
      const count = event.data as number;
      revealCount = count;
    });
  });
</script>

{revealCount}