<script lang="ts">
  import type { Config } from "$lib/config";
  import { MULTI_PROVIDERS } from "$lib/config";
  import { Label } from "./ui/label";
  import * as Select from "./ui/select";
  import { Switch } from "./ui/switch";

  export let config: Config | null = null;
  export let onChange: (config: Config) => void = () => {};

  function updateConfig(patch: Partial<Config>) {
    if (!config) return;
    onChange({ ...config, ...patch });
  }
</script>

<div class="flex gap-5 items-center">
  <div>
    <Label for="multi-provider">Multi Link Website</Label>
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
      <Select.Trigger id="multi-provider" class="w-[180px]">
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

  <div class="flex flex-col gap-3">
    <div class="flex items-center space-x-2">
      <Switch
        checked={config?.autoOpen ?? false}
        disabled={!config}
        id="auto-open"
        onCheckedChange={(autoOpen) => updateConfig({ autoOpen })}
      />
      <Label for="auto-open">Auto Open Multi</Label>
    </div>
    <div class="flex items-center space-x-2">
      <Switch
        checked={config?.autoAccept ?? false}
        disabled={!config}
        id="auto-accept"
        onCheckedChange={(autoAccept) => updateConfig({ autoAccept })}
      />
      <Label for="auto-accept">Auto Accept</Label>
    </div>
  </div>
</div>
