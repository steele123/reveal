<script lang="ts">
    import { fade } from "svelte/transition";

    export let connected = false;
</script>

<div class="px-4 items-center flex pt-1 border-t">
    {#if connected}
      <div in:fade={{ duration: 2000 }} class="flex items-center gap-1 text-sm">
        <div class="bg-green-500 animate-pulse h-2 w-2 rounded-full" />
        <div>Connected to League Client</div>
      </div>
    {:else}
      <div in:fade={{ duration: 2000 }} class="flex items-center gap-1 text-sm">
        <div class="bg-red-500 animate-pulse h-2 w-2 rounded-full" />
        <div>Not Connected to League Client</div>
      </div>
    {/if}
    <a
      href="https://hyeb.gg/reveal"
      target="_blank"
      class="text-blue-500 ml-auto hover:underline text-xs">hyeb.gg/reveal</a
    >
  </div>