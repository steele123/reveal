<script lang="ts">
  import type { Config } from "$lib/config";
  import { MULTI_PROVIDERS, PLAYER_CLICK_ACTIONS } from "$lib/config";
  import { logFrontendError, openLogFolder } from "$lib/logging";
  import { isTauriRuntime } from "$lib/runtime";
  import { Label } from "./ui/label";
  import * as Select from "./ui/select";
  import { Switch } from "./ui/switch";

  export let config: Config | null = null;
  export let onChange: (config: Config) => void = () => {};

  const autoOpenWaitOptions = [
    { label: "Now", value: "0" },
    { label: "4 sec", value: "4" },
    { label: "6 sec", value: "6" },
    { label: "10 sec", value: "10" },
  ];

  function labelOf(options: readonly { label: string; value: string }[], value: string): string {
    return options.find((option) => option.value === value)?.label ?? value;
  }

  function updateConfig(patch: Partial<Config>) {
    if (!config) return;
    onChange({ ...config, ...patch });
  }

  function openLogs() {
    if (!isTauriRuntime()) return;
    openLogFolder().catch((error) => logFrontendError("Failed to open the log folder", error));
  }
</script>

<section class="grid grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)] gap-3">
  <div class="reveal-panel p-3">
    <Label
      for="multi-provider"
      class="mb-1.5 block text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground"
    >
      Multi-search
    </Label>
    <Select.Root
      type="single"
      items={[...MULTI_PROVIDERS]}
      value={config?.multiProvider ?? "opgg"}
      onValueChange={(value) => {
        const provider = MULTI_PROVIDERS.find((provider) => provider.value === value);
        if (provider) updateConfig({ multiProvider: provider.value });
      }}
    >
      <Select.Trigger
        id="multi-provider"
        class="h-9 w-full border-white/10 bg-white/[0.035] shadow-none"
      >
        {labelOf(MULTI_PROVIDERS, config?.multiProvider ?? "opgg")}
      </Select.Trigger>
      <Select.Content>
        {#each MULTI_PROVIDERS as provider}
          <Select.Item value={provider.value} label={provider.label} />
        {/each}
      </Select.Content>
    </Select.Root>

    <Label
      for="player-click-action"
      class="mb-1.5 mt-3 block text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground"
    >
      Clicking a player
    </Label>
    <Select.Root
      type="single"
      items={[...PLAYER_CLICK_ACTIONS]}
      value={config?.playerClickAction ?? "details"}
      onValueChange={(value) => {
        const action = PLAYER_CLICK_ACTIONS.find((action) => action.value === value);
        if (action) updateConfig({ playerClickAction: action.value });
      }}
    >
      <Select.Trigger
        id="player-click-action"
        class="h-9 w-full border-white/10 bg-white/[0.035] shadow-none"
      >
        {labelOf(PLAYER_CLICK_ACTIONS, config?.playerClickAction ?? "details")}
      </Select.Trigger>
      <Select.Content>
        {#each PLAYER_CLICK_ACTIONS as action}
          <Select.Item value={action.value} label={action.label} />
        {/each}
      </Select.Content>
    </Select.Root>

    <button
      type="button"
      class="mt-2.5 text-[10px] text-muted-foreground underline-offset-2 transition hover:text-foreground hover:underline"
      title="Opens the folder with reveal.log. Attach it when you report a bug."
      on:click={openLogs}
    >
      Open logs folder
    </button>
  </div>

  <div class="reveal-panel divide-y divide-white/10 overflow-hidden p-1">
    <div
      class="flex items-center gap-3 rounded-lg px-3 py-1.5 transition hover:bg-white/[0.025]"
    >
      <Label for="auto-open" class="min-w-0 flex-1 cursor-pointer">
        <span class="block text-xs font-medium">Open lookup automatically</span>
        <span class="mt-0.5 block text-[10px] text-muted-foreground">
          Wait for all five names, up to
        </span>
      </Label>
      <div class="flex shrink-0 items-center gap-2">
        <Select.Root
          type="single"
          disabled={!config?.autoOpen}
          items={autoOpenWaitOptions}
          value={String(config?.autoOpenDelaySeconds ?? 6)}
          onValueChange={(value) => updateConfig({ autoOpenDelaySeconds: Number(value) })}
        >
          <Select.Trigger
            aria-label="Maximum auto-open wait"
            class="h-7 w-[72px] border-white/10 bg-white/[0.035] px-2 text-[10px] shadow-none"
          >
            {labelOf(autoOpenWaitOptions, String(config?.autoOpenDelaySeconds ?? 6))}
          </Select.Trigger>
          <Select.Content>
            {#each autoOpenWaitOptions as option}
              <Select.Item value={option.value} label={option.label} />
            {/each}
          </Select.Content>
        </Select.Root>
        <Switch
          checked={config?.autoOpen ?? false}
          disabled={!config}
          id="auto-open"
          onCheckedChange={(autoOpen) => updateConfig({ autoOpen })}
        />
      </div>
    </div>
    <div
      class="flex items-center gap-3 rounded-lg px-3 py-1.5 transition hover:bg-white/[0.025]"
    >
      <Label for="auto-accept" class="min-w-0 flex-1 cursor-pointer">
        <span class="block text-xs font-medium">Accept matches automatically</span>
        <span class="mt-0.5 block text-[10px] text-muted-foreground">
          When League shows a ready check
        </span>
      </Label>
      <Switch
        checked={config?.autoAccept ?? false}
        disabled={!config}
        id="auto-accept"
        onCheckedChange={(autoAccept) => updateConfig({ autoAccept })}
      />
    </div>
    <div
      class="flex items-center gap-3 rounded-lg px-3 py-1.5 transition hover:bg-white/[0.025]"
    >
      <Label for="pause-after-dodge" class="min-w-0 flex-1 cursor-pointer">
        <span class="block text-xs font-medium">Pause queue after a dodge</span>
        <span class="mt-0.5 block text-[10px] text-muted-foreground">
          Return to lobby until you queue again
        </span>
      </Label>
      <Switch
        checked={config?.pauseQueueAfterDodge ?? false}
        disabled={!config}
        id="pause-after-dodge"
        onCheckedChange={(pauseQueueAfterDodge) => updateConfig({ pauseQueueAfterDodge })}
      />
    </div>
  </div>
</section>
