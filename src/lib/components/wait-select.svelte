<script lang="ts">
  import * as Select from "./ui/select";

  /** Seconds to wait, as the select's string values. */
  export let options: { label: string; value: string }[];
  export let seconds: number;
  export let label: string;
  export let disabled = false;
  export let onChange: (seconds: number) => void = () => {};

  $: value = String(seconds);
</script>

<Select.Root
  type="single"
  {disabled}
  items={options}
  {value}
  onValueChange={(next) => onChange(Number(next))}
>
  <Select.Trigger
    aria-label={label}
    class="h-7 w-[92px] border-white/10 bg-white/[0.035] px-2 text-[10px] shadow-none"
  >
    {options.find((option) => option.value === value)?.label ?? `After ${value} sec`}
  </Select.Trigger>
  <Select.Content>
    {#each options as option}
      <Select.Item value={option.value} label={option.label} />
    {/each}
  </Select.Content>
</Select.Root>
