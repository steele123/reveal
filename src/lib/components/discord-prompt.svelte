<script lang="ts">
  import { fade, scale } from "svelte/transition";

  export let onClose: (joined: boolean) => void = () => {};

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") onClose(false);
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div
  class="absolute inset-0 z-50 grid place-items-center bg-black/60 p-6 backdrop-blur-sm"
  transition:fade={{ duration: 150 }}
>
  <div
    role="dialog"
    aria-modal="true"
    aria-labelledby="discord-prompt-title"
    class="w-full max-w-sm rounded-xl border border-white/10 bg-[#0b1220] p-5 text-center shadow-2xl"
    transition:scale={{ duration: 150, start: 0.96 }}
  >
    <div
      class="mx-auto grid h-12 w-12 place-items-center rounded-2xl bg-[#5865F2]/15 text-[#8b95f7]"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="24"
        height="24"
        viewBox="0 0 24 24"
        fill="currentColor"
        aria-hidden="true"
        ><path
          d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"
        /></svg
      >
    </div>
    <h2 id="discord-prompt-title" class="mt-4 text-base font-semibold">
      Join the Reveal Discord
    </h2>
    <p class="mt-1.5 text-xs leading-relaxed text-muted-foreground">
      Get update news, report bugs, and suggest features. You'll only see this
      once; the Discord link stays in the footer.
    </p>
    <div class="mt-5 flex gap-2">
      <button
        class="h-9 flex-1 rounded-md border border-white/10 bg-white/[0.035] text-xs font-medium text-muted-foreground transition hover:bg-white/[0.07] hover:text-foreground"
        on:click={() => onClose(false)}
      >
        Not now
      </button>
      <button
        class="h-9 flex-1 rounded-md bg-[#5865F2] text-xs font-semibold text-white shadow-[0_8px_24px_rgba(88,101,242,0.25)] transition hover:bg-[#6d78f4]"
        on:click={() => onClose(true)}
      >
        Join Discord
      </button>
    </div>
  </div>
</div>
