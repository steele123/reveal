<script lang="ts">
  import { fade } from "svelte/transition";

  export let connected = false;
</script>

<div class="flex flex-col">
  <div class="text-xs border-t px-4 bg-primary-foreground py-2 text-muted-foreground">Need League of Legends smurfs? <a target="_blank" class="text-blue-500 hover:underline" href="https://mao.market">mao.market</a></div>
  <div class="px-4 pt-1.5 items-center flex border-t">
    {#if connected}
      <div
        in:fade={{ duration: 2000 }}
        class="flex items-center mb-auto gap-2 text-sm"
      >
        <div class="bg-green-500 animate-pulse h-2 w-2 rounded-full" />
        <div>Connected to League Client</div>
      </div>
    {:else}
      <div
        in:fade={{ duration: 2000 }}
        class="flex items-center mb-auto gap-2 text-sm"
      >
        <div class="bg-red-500 animate-pulse h-2 w-2 text-sm rounded-full" />
        <div>Can't Connect to League Client</div>
      </div>
    {/if}
    <div class="flex gap-1 text-xs ml-auto flex-col">
      <a
        href="https://hyeb.gg/reveal"
        target="_blank"
        class="flex items-center border px-4 bg-primary-foreground rounded gap-2"
      >
        <div class="text-blue-500 hover:underline">hyeb.gg/reveal</div>
      </a>
    </div>
  </div>
</div>
