<script lang="ts">
  import { fade } from "svelte/transition";

  export let connected = false;
</script>

<div class="px-2 pt-1 items-center flex border-t">
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
      class="flex items-center border pl-4 pr-2 bg-primary-foreground rounded gap-2"
    >
      <div class="text-blue-500 hover:underline">hyeb.gg/reveal</div>
    </a>
  </div>
</div>
