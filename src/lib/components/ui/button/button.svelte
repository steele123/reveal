<script lang="ts">
	import type { HTMLButtonAttributes } from "svelte/elements";
	import { cn } from "$lib/utils";
	import { buttonVariants, type ButtonSize, type ButtonVariant } from "./variants";

	let {
		class: className,
		variant = "default",
		size = "default",
		type = "button",
		children,
		...rest
	}: HTMLButtonAttributes & { variant?: ButtonVariant; size?: ButtonSize } = $props();
</script>

<button {type} class={cn(buttonVariants({ variant, size }), className)} {...rest}>
	{@render children?.()}
</button>
