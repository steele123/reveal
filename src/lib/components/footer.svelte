<script lang="ts">
  import { fade } from "svelte/transition";

  export let connected = false;
</script>

<div class="px-4 justify-center items-center flex border-t h-[40px]">
  <div>
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
  </div>
  <div class="flex gap-2 text-xs ml-auto">
    <a
      href="https://hyperboost.gg/reveal"
      target="_blank"
      class="flex items-center group border px-1 py-1 bg-primary-foreground rounded gap-2"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="17"
        height="17"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="lucide lucide-external-link group-hover:text-blue-500 transition ease-in-out duration-300"
        ><path d="M15 3h6v6" /><path d="M10 14 21 3" /><path
          d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"
        /></svg
      >
    </a>
  </div>
</div>
