<script lang="ts">
  import { fade } from "svelte/transition";

  export let connected = false;
</script>

<div class="px-4 items-center flex py-2 border-t">
  {#if connected}
    <div
      in:fade={{ duration: 2000 }}
      class="flex items-center mb-auto gap-1 text-sm"
    >
      <div class="bg-green-500 animate-pulse h-2 w-2 rounded-full" />
      <div>Connected to League Client</div>
    </div>
  {:else}
    <div
      in:fade={{ duration: 2000 }}
      class="flex items-center mb-auto gap-1 text-sm"
    >
      <div class="bg-red-500 animate-pulse h-2 w-2 text-sm rounded-full" />
      <div>Can't Connect to League Client</div>
    </div>
  {/if}
  <div class="flex gap-1 text-xs ml-auto flex-col">
    <div class="flex gap-2">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="15"
        height="15"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="lucide lucide-external-link"
        ><path d="M15 3h6v6" /><path d="M10 14 21 3" /><path
          d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"
        /></svg
      >
      <a
        href="https://hyeb.gg/reveal"
        target="_blank"
        class="text-blue-500 hover:underline">hyeb.gg/reveal</a
      >
    </div>
    <div class="flex gap-2">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="15"
        height="15"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="lucide lucide-mail"
        ><rect width="20" height="16" x="2" y="4" rx="2" /><path
          d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"
        /></svg
      >
      <a href="mailto:steele@catlab.gg" class="text-blue-500 hover:underline">
        steele@catlab.gg
      </a>
    </div>
  </div>
</div>
