<script lang="ts">
	import { Select as SelectPrimitive } from "bits-ui";
	import { cn } from "$lib/utils";

	let {
		class: className,
		sideOffset = 4,
		children,
		...rest
	}: SelectPrimitive.ContentProps = $props();
</script>

<SelectPrimitive.Portal>
	<SelectPrimitive.Content
		{sideOffset}
		class={cn(
			"relative z-50 min-w-[var(--bits-select-anchor-width)] overflow-hidden rounded-md border bg-popover text-popover-foreground shadow-md focus:outline-none",
			className
		)}
		{...rest}
	>
		<SelectPrimitive.Viewport class="w-full p-1">
			{@render children?.()}
		</SelectPrimitive.Viewport>
	</SelectPrimitive.Content>
</SelectPrimitive.Portal>
