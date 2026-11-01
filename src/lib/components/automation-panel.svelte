<script lang="ts">
  import { onMount } from "svelte";
  import type { Config } from "$lib/config";
  import { formatAssignedPosition } from "$lib/champ_select";
  import { loadChampionNames, type ChampionName } from "$lib/champions";
  import { logFrontendError } from "$lib/logging";
  import { LANES, type Role } from "$lib/matchups";
  import ChampionList from "./champion-list.svelte";
  import InfoTip from "./info-tip.svelte";
  import NotificationSettings from "./notification-settings.svelte";
  import TimedToggle from "./timed-toggle.svelte";
  import { Label } from "./ui/label";

  export let config: Config | null = null;
  export let onChange: (config: Config) => void = () => {};

  let champions: ChampionName[] = [];
  let championsFailed = false;
  /** Opens on the first role with picks, which is usually the one you play. */
  let role: Role = LANES.find((lane) => config?.pickChampions[lane]?.length) ?? LANES[0];

  /** How each setting decides what to do, for its info icon. */
  const TIPS = {
    autoAccept:
      "When the ready check pops, Reveal waits the time you pick (counted from League's own 10-second timer), then accepts it. Last second accepts at 9 seconds, leaving a second for the click to land. If you accept or decline it yourself while it waits, Reveal leaves it alone. It also leaves it alone while the queue is paused after a dodge.",
    autoRequeue:
      "When the game ends and the client reaches the post-game screen, Reveal waits the time you pick (counting down at the top of the Reveal tab), clicks Play Again and starts the same queue. If by then you've queued yourself, left the lobby or the queue is paused after a dodge, it does nothing. Only the party leader can start a queue.",
    requeuePastEstimate:
      "While you're in queue, Reveal compares your time in queue with League's estimated time every 2 seconds. Once you've waited half again as long as the estimate (and at least a minute past it), it checks once more that no match has been found, leaves the queue, waits the time you pick and queues again. It never touches a match that's already been found or a ready check that's up.",
    pauseAfterDodge:
      "If champ select ends without the game starting (someone dodged), Reveal stops League from queueing you again and keeps you in the lobby until you press Find Match yourself, or until the time you pick has passed, when Reveal queues again (counting down at the top of the Reveal tab). The automatic queue options wait too.",
    autoOpen:
      "In champ select, Reveal opens your multi-search site (set on the Reveal tab) with your teammates' names as soon as all five are revealed, or after the wait you pick here with whoever is known by then.",
    autoBan:
      "In your ban turn, once only the time you set is left (the last second, by default) and you haven't banned: the champion you're hovering is banned, or else the first champion from this list that's still bannable and isn't hovered or picked by a teammate. If none qualifies, Reveal does nothing.",
    autoPick:
      "In your pick turn, once only the time you set is left (the last second, by default) and you haven't locked in: Reveal locks in your hover (or the champion you declared during planning), or else the first champion from your assigned role's list that you own and that isn't banned, picked or hovered by a teammate. If none qualifies, it does nothing and League's own timer runs out.",
  };

  // Times for each on/off setting; `short` shows in the control, `label` in its menu.
  const acceptTimes = [
    { value: 0, label: "Right away", short: "0s" },
    { value: 2000, label: "After 2 sec", short: "2s" },
    { value: 4000, label: "After 4 sec", short: "4s" },
    { value: 6000, label: "After 6 sec", short: "6s" },
    { value: 8000, label: "After 8 sec", short: "8s" },
    { value: 9000, label: "Last second", short: "Last" },
  ];
  const requeueTimes = [
    { value: 0, label: "Right away", short: "0s" },
    { value: 10, label: "After 10 sec", short: "10s" },
    { value: 20, label: "After 20 sec", short: "20s" },
    { value: 30, label: "After 30 sec", short: "30s" },
    { value: 60, label: "After 60 sec", short: "60s" },
  ];
  const dodgeTimes = [
    { value: 0, label: "Until I queue", short: "∞" },
    { value: 10, label: "10 sec", short: "10s" },
    { value: 30, label: "30 sec", short: "30s" },
    { value: 60, label: "60 sec", short: "60s" },
    { value: 120, label: "2 min", short: "2m" },
  ];
  const openTimes = [
    { value: 0, label: "Now", short: "0s" },
    { value: 4, label: "Up to 4 sec", short: "4s" },
    { value: 6, label: "Up to 6 sec", short: "6s" },
    { value: 10, label: "Up to 10 sec", short: "10s" },
  ];

  /** When to ban or lock in, as seconds left on League's timer. */
  const secondsLeftTimes = [
    { value: 1, label: "Last second", short: "Last" },
    { value: 3, label: "3 sec left", short: "3s" },
    { value: 5, label: "5 sec left", short: "5s" },
    { value: 10, label: "10 sec left", short: "10s" },
  ];

  onMount(() => {
    loadChampionNames()
      .then((list) => (champions = list))
      .catch((error) => {
        championsFailed = true;
        logFrontendError("Failed to load the champion list", error);
      });
  });

  function update(patch: Partial<Config>) {
    if (!config) return;
    onChange({ ...config, ...patch });
  }

  function setPicks(ids: number[]) {
    if (!config) return;
    update({ pickChampions: { ...config.pickChampions, [role]: ids } });
  }
</script>

<div class="reveal-scrollbar -mr-1 flex h-full min-h-0 flex-col gap-3 overflow-y-auto pr-1">
  <section class="reveal-panel shrink-0 divide-y divide-white/10 p-1">
    <div class="px-3 pb-1.5 pt-2 text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">
      Queue
    </div>
    <div class="flex items-center gap-3 rounded-lg px-3 py-2">
      <Label for="auto-accept" class="min-w-0 flex-1 cursor-pointer">
        <span class="block text-xs font-medium">Accept matches automatically<InfoTip text={TIPS.autoAccept} /></span>
        <span class="mt-0.5 block text-[10px] text-muted-foreground">
          Waits first, so you can still decline it yourself
        </span>
      </Label>
      <TimedToggle
        label="Accept matches automatically"
        id="auto-accept"
        enabled={config?.autoAccept ?? false}
        value={config?.acceptDelay ?? 2000}
        options={acceptTimes}
        disabled={!config}
        onToggle={(autoAccept) => update({ autoAccept })}
        onChange={(acceptDelay) => update({ acceptDelay })}
      />
    </div>
    <div class="flex items-center gap-3 rounded-lg px-3 py-2">
      <Label for="auto-requeue" class="min-w-0 flex-1 cursor-pointer">
        <span class="block text-xs font-medium">Queue again after a game<InfoTip text={TIPS.autoRequeue} /></span>
        <span class="mt-0.5 block text-[10px] text-muted-foreground">
          Once the game ends, back to the lobby and into the same queue
        </span>
      </Label>
      <TimedToggle
        label="Queue again after a game"
        id="auto-requeue"
        enabled={config?.autoRequeue ?? false}
        value={config?.requeueDelaySeconds ?? 0}
        options={requeueTimes}
        disabled={!config}
        onToggle={(autoRequeue) => update({ autoRequeue })}
        onChange={(requeueDelaySeconds) => update({ requeueDelaySeconds })}
      />
    </div>
    <div class="flex items-center gap-3 rounded-lg px-3 py-2">
      <Label for="requeue-past-estimate" class="min-w-0 flex-1 cursor-pointer">
        <span class="block text-xs font-medium">Restart long queues<InfoTip text={TIPS.requeuePastEstimate} /></span>
        <span class="mt-0.5 block text-[10px] text-muted-foreground">
          Once you've waited well past League's estimated time, leave the queue and rejoin
        </span>
      </Label>
      <TimedToggle
        label="Restart long queues"
        id="requeue-past-estimate"
        enabled={config?.requeuePastEstimate ?? false}
        value={config?.longQueueRequeueDelaySeconds ?? 0}
        options={requeueTimes}
        disabled={!config}
        onToggle={(requeuePastEstimate) => update({ requeuePastEstimate })}
        onChange={(longQueueRequeueDelaySeconds) => update({ longQueueRequeueDelaySeconds })}
      />
    </div>
    <div class="flex items-center gap-3 rounded-lg px-3 py-2">
      <Label for="pause-after-dodge" class="min-w-0 flex-1 cursor-pointer">
        <span class="block text-xs font-medium">Pause queue after a dodge<InfoTip text={TIPS.pauseAfterDodge} /></span>
        <span class="mt-0.5 block text-[10px] text-muted-foreground">
          Return to the lobby and wait before queueing again; the automatic requeues wait too
        </span>
      </Label>
      <TimedToggle
        label="Pause queue after a dodge"
        id="pause-after-dodge"
        enabled={config?.pauseQueueAfterDodge ?? false}
        value={config?.dodgePauseSeconds ?? 0}
        options={dodgeTimes}
        disabled={!config}
        onToggle={(pauseQueueAfterDodge) => update({ pauseQueueAfterDodge })}
        onChange={(dodgePauseSeconds) => update({ dodgePauseSeconds })}
      />
    </div>
  </section>

  <section class="reveal-panel shrink-0 divide-y divide-white/10 p-1">
    <div class="px-3 pb-1.5 pt-2">
      <div class="text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground">
        Champ Select
      </div>
    </div>
    <div class="flex items-center gap-3 px-3 py-2">
      <Label for="auto-open" class="min-w-0 flex-1 cursor-pointer">
        <span class="block text-xs font-medium">Open lookup automatically<InfoTip text={TIPS.autoOpen} /></span>
        <span class="mt-0.5 block text-[10px] text-muted-foreground">
          Opens your multi-search site once all five names are in, or after the wait
        </span>
      </Label>
      <div class="flex shrink-0 items-center gap-2">
        <TimedToggle
          label="Open lookup automatically"
          id="auto-open"
          enabled={config?.autoOpen ?? false}
          value={config?.autoOpenDelaySeconds ?? 6}
          options={openTimes}
          disabled={!config}
          onToggle={(autoOpen) => update({ autoOpen })}
          onChange={(autoOpenDelaySeconds) => update({ autoOpenDelaySeconds })}
        />
      </div>
    </div>
    <div class="flex flex-col gap-2 px-3 py-2">
      <div class="flex items-center gap-3">
        <Label for="auto-ban" class="min-w-0 flex-1 cursor-pointer">
          <span class="block text-xs font-medium">Ban automatically<InfoTip text={TIPS.autoBan} /></span>
          <span class="mt-0.5 block text-[10px] text-muted-foreground">
            Your hover, or else the first available champion from this list
          </span>
        </Label>
        <TimedToggle
          id="auto-ban"
          label="Ban automatically"
          enabled={config?.autoBan ?? false}
          value={config?.autoBanSecondsLeft ?? 1}
          options={secondsLeftTimes}
          disabled={!config}
          onToggle={(autoBan) => update({ autoBan })}
          onChange={(autoBanSecondsLeft) => update({ autoBanSecondsLeft })}
        />
      </div>
      <ChampionList
        id="ban-champions"
        ids={config?.banChampions ?? []}
        {champions}
        disabled={!config?.autoBan}
        placeholder="Add bans"
        onChange={(banChampions) => update({ banChampions })}
      />
    </div>
    <div class="flex flex-col gap-2 px-3 py-2">
      <div class="flex items-center gap-3">
        <Label for="auto-pick" class="min-w-0 flex-1 cursor-pointer">
          <span class="block text-xs font-medium">Lock in automatically<InfoTip text={TIPS.autoPick} /></span>
          <span class="mt-0.5 block text-[10px] text-muted-foreground">
            Your hover, or else the first available champion for the role you got
          </span>
        </Label>
        <TimedToggle
          id="auto-pick"
          label="Lock in automatically"
          enabled={config?.autoPick ?? false}
          value={config?.autoPickSecondsLeft ?? 1}
          options={secondsLeftTimes}
          disabled={!config}
          onToggle={(autoPick) => update({ autoPick })}
          onChange={(autoPickSecondsLeft) => update({ autoPickSecondsLeft })}
        />
      </div>
      <div
        class="grid grid-cols-5 gap-0.5 rounded-lg border border-white/10 bg-white/[0.025] p-0.5"
        role="tablist"
        aria-label="Role"
      >
        {#each LANES as lane (lane)}
          {@const count = config?.pickChampions[lane]?.length ?? 0}
          <button
            type="button"
            role="tab"
            aria-selected={role === lane}
            class="flex items-center justify-center gap-1.5 rounded-md px-2 py-1 text-[10px] font-medium transition focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-blue-400 {role ===
            lane
              ? 'bg-blue-500/15 text-blue-200 shadow-sm'
              : 'text-muted-foreground hover:bg-white/[0.04] hover:text-foreground'}"
            on:click={() => (role = lane)}
          >
            {formatAssignedPosition(lane)}
            <span
              class="min-w-[16px] rounded-full px-1 text-[9px] tabular-nums leading-[14px] {count
                ? 'bg-white/10 text-foreground'
                : 'text-muted-foreground/60'}"
            >
              {count}
            </span>
          </button>
        {/each}
      </div>
      {#key role}
        <ChampionList
          id={`pick-champions-${role}`}
          ids={config?.pickChampions[role] ?? []}
          {champions}
          disabled={!config?.autoPick}
          placeholder={`Add ${formatAssignedPosition(role)} picks`}
          onChange={setPicks}
        />
      {/key}
    </div>
  </section>

  <NotificationSettings {config} {onChange} />

  {#if championsFailed}
    <div class="text-[10px] text-red-300">
      Couldn't load the champion list from Community Dragon, so names can't be searched right now.
    </div>
  {/if}
</div>
