<script lang="ts">
  import { onDestroy } from "svelte";
  import { fade } from "svelte/transition";

  /** When Reveal queues again, in epoch milliseconds. */
  export let deadline: number;
  /** What the wait follows: a game, leaving a queue that ran long, or a dodge. */
  export let after: "game" | "queue" | "dodge";

  let now = Date.now();
  const timer = setInterval(() => (now = Date.now()), 250);
  onDestroy(() => clearInterval(timer));

  $: seconds = Math.max(0, Math.ceil((deadline - now) / 1000));
</script>

{#if seconds > 0}
  <div
    in:fade
    role="status"
    class="reveal-panel flex shrink-0 items-center gap-3 border-emerald-400/20 bg-emerald-500/[0.06] px-4 py-2.5"
  >
    <span class="w-8 text-center text-lg font-semibold tabular-nums text-emerald-300">{seconds}</span>
    <div class="min-w-0 text-xs">
      <div class="font-medium">
        {seconds === 1 ? "1 second" : `${seconds} seconds`} to requeue
      </div>
      <div class="mt-0.5 text-[10px] text-muted-foreground">
        {after === "game"
          ? "Reveal will press Play Again and start the same queue."
          : after === "queue"
            ? "Your queue ran long; Reveal will start it again."
            : "The queue is paused after a dodge; Reveal will start it again."}
      </div>
    </div>
  </div>
{/if}
