<script lang="ts">
  import { Popover } from "bits-ui";
  import { DISCORD_URL } from "$lib/link_actions";
  import { logFrontendError, openLogFolder } from "$lib/logging";
  import { isTauriRuntime } from "$lib/runtime";

  export let connected = false;
  export let inGame = false;
  /** League's gameflow phase, such as "Lobby" or "ChampSelect". */
  export let state = "Unknown";
  export let queuePaused = false;

  $: status = describe(state, connected, inGame, queuePaused);

  function describe(state: string, connected: boolean, inGame: boolean, queuePaused: boolean) {
    if (inGame || state === "InProgress")
      return {
        title: "Game in progress",
        dot: "bg-blue-400 shadow-[0_0_8px_rgba(96,165,250,0.7)]",
        detail: "Reveal is standing by and will be ready for your next lobby.",
      };
    if (!connected)
      return {
        title: "Looking for League",
        dot: "bg-amber-400",
        detail:
          "Start League, or restart it and run Reveal as administrator if it is already open.",
      };
    if (queuePaused)
      return {
        title: "Queue paused after a dodge",
        dot: "bg-amber-400",
        detail: "Reveal stopped the next search. Queue again in League when you're ready.",
      };
    if (state === "ChampSelect")
      return {
        title: "In Champ Select",
        dot: "bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.7)]",
        detail: "Your teammates' names and the enemy counters are on the Reveal tab.",
      };
    return {
      title: "Ready for Champ Select",
      dot: "bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.7)]",
      detail: "Join a lobby and queue normally. Teammate names will appear automatically.",
    };
  }

  function formatState(value: string): string {
    if (value === "InProgress") return "In game";
    if (value === "ChampSelect") return "Champ Select";
    if (value === "Unknown" || value === "None") return "Not in a lobby";
    return value.replace(/([a-z])([A-Z])/g, "$1 $2");
  }

  function openLogs() {
    if (!isTauriRuntime()) return;
    openLogFolder().catch((error) => logFrontendError("Failed to open the log folder", error));
  }
</script>

<footer
  class="flex h-10 shrink-0 items-center border-t border-white/10 px-5 text-xs"
>
  <Popover.Root>
    <Popover.Trigger
      class="-ml-2 flex min-w-0 items-center gap-2 rounded-md px-2 py-1 text-muted-foreground transition hover:bg-white/5 hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-blue-400 data-[state=open]:bg-white/5 data-[state=open]:text-foreground"
    >
      <span class={`h-1.5 w-1.5 shrink-0 rounded-full ${status.dot}`}></span>
      <span class="truncate">{status.title}</span>
    </Popover.Trigger>
    <Popover.Portal>
      <Popover.Content
        side="top"
        align="start"
        sideOffset={8}
        collisionPadding={8}
        class="z-50 w-[280px] rounded-lg border border-white/10 bg-[#0f1726] p-3 text-xs shadow-2xl focus:outline-none"
      >
        <div class="font-semibold">{status.title}</div>
        <p class="mt-1 leading-relaxed text-muted-foreground">{status.detail}</p>
        <dl class="mt-3 space-y-1.5 border-t border-white/10 pt-2.5 text-[11px]">
          <div class="flex items-center justify-between gap-2">
            <dt class="text-muted-foreground">League</dt>
            <dd class:text-emerald-300={connected} class:text-amber-300={!connected}>
              {connected ? "Connected" : "Unavailable"}
            </dd>
          </div>
          <div class="flex items-center justify-between gap-2">
            <dt class="text-muted-foreground">Client state</dt>
            <dd>{connected ? formatState(state) : "—"}</dd>
          </div>
        </dl>
      </Popover.Content>
    </Popover.Portal>
  </Popover.Root>
  <div class="ml-auto flex items-center gap-1">
    <button
      type="button"
      class="rounded-md px-2 py-1 text-muted-foreground transition hover:bg-white/5 hover:text-foreground"
      title="Opens the folder with reveal.log. Attach it when you report a bug."
      on:click={openLogs}
    >
      Logs
    </button>
    <a
      href={DISCORD_URL}
      target="_blank"
      rel="noreferrer"
      class="group flex items-center gap-1.5 rounded-md px-2 py-1 text-muted-foreground transition hover:bg-white/5 hover:text-foreground"
    >
      Discord
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="13"
        height="13"
        viewBox="0 0 24 24"
        fill="currentColor"
        aria-hidden="true"
        class="transition group-hover:text-[#5865F2]"
        ><path
          d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"
        /></svg
      >
    </a>
    <a
      href="https://lobbyreveal.app/reveal"
      target="_blank"
      rel="noreferrer"
      class="group flex items-center gap-1.5 rounded-md px-2 py-1 text-muted-foreground transition hover:bg-white/5 hover:text-foreground"
    >
      Website
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="13"
        height="13"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="transition group-hover:text-blue-400"
        ><path d="M15 3h6v6" /><path d="M10 14 21 3" /><path
          d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"
        /></svg
      >
    </a>
  </div>
</footer>
