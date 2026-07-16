<script lang="ts">
  import type { Config } from "$lib/config";
  import { MULTI_PROVIDERS } from "$lib/config";
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

  function updateConfig(patch: Partial<Config>) {
    if (!config) return;
    onChange({ ...config, ...patch });
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
      onSelectedChange={(selected) => {
        const provider = MULTI_PROVIDERS.find(
          ({ value }) => value === selected?.value,
        );
        if (provider) updateConfig({ multiProvider: provider.value });
      }}
      selected={MULTI_PROVIDERS.find(
        ({ value }) => value === config?.multiProvider,
      )}
    >
      <Select.Trigger
        id="multi-provider"
        class="h-9 w-full border-white/10 bg-white/[0.035] shadow-none"
      >
        <Select.Value />
      </Select.Trigger>
      <Select.Content>
        <Select.Group>
          {#each MULTI_PROVIDERS as provider}
            <Select.Item value={provider.value} label={provider.label}>
              {provider.label}
            </Select.Item>
          {/each}
        </Select.Group>
      </Select.Content>
    </Select.Root>
  </div>

  <div class="reveal-panel divide-y divide-white/10 overflow-hidden p-1">
    <div
      class="flex items-center gap-3 rounded-lg px-3 py-2 transition hover:bg-white/[0.025]"
    >
      <Label for="auto-open" class="min-w-0 flex-1 cursor-pointer">
        <span class="block text-xs font-medium">Open lookup automatically</span>
        <span class="mt-0.5 block text-[10px] text-muted-foreground">
          Wait for all five names, up to
        </span>
      </Label>
      <div class="flex shrink-0 items-center gap-2">
        <Select.Root
          disabled={!config?.autoOpen}
          onSelectedChange={(selected) => {
            if (!selected) return;
            updateConfig({ autoOpenDelaySeconds: Number(selected.value) });
          }}
          selected={autoOpenWaitOptions.find(
            ({ value }) => Number(value) === config?.autoOpenDelaySeconds,
          )}
        >
          <Select.Trigger
            aria-label="Maximum auto-open wait"
            class="h-7 w-[72px] border-white/10 bg-white/[0.035] px-2 text-[10px] shadow-none"
          >
            <Select.Value />
          </Select.Trigger>
          <Select.Content>
            <Select.Group>
              {#each autoOpenWaitOptions as option}
                <Select.Item value={option.value} label={option.label}>
                  {option.label}
                </Select.Item>
              {/each}
            </Select.Group>
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
      class="flex items-center gap-3 rounded-lg px-3 py-2 transition hover:bg-white/[0.025]"
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
  </div>
</section>
