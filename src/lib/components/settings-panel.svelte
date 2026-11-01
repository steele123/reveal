<script lang="ts">
  import type { Config } from "$lib/config";
  import { MULTI_PROVIDERS, PLAYER_CLICK_ACTIONS } from "$lib/config";
  import { Label } from "./ui/label";
  import * as Select from "./ui/select";

  export let config: Config | null = null;
  export let onChange: (config: Config) => void = () => {};

  function labelOf(options: readonly { label: string; value: string }[], value: string): string {
    return options.find((option) => option.value === value)?.label ?? value;
  }

  function updateConfig(patch: Partial<Config>) {
    if (!config) return;
    onChange({ ...config, ...patch });
  }
</script>

<section class="reveal-panel grid grid-cols-2 gap-x-3 p-3">
  <div>
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

  </div>
  <div>
    <Label
      for="player-click-action"
      class="mb-1.5 block text-[11px] font-medium uppercase tracking-[0.12em] text-muted-foreground"
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
  </div>

</section>
