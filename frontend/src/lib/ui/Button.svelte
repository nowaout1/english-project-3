<script lang="ts">
	import { tv } from 'tailwind-variants';
	import type { Snippet } from 'svelte';
	import { twMerge } from 'tailwind-merge';

	const {
		children,
		className,
		href,
		variant,
		size,
		...attr
	}: {
		children: Snippet;
		className?: string;
		href?: string;
		disabled?: boolean;
		variant?: 'primary' | 'outline' | 'secondary' | 'destructive';
		size?: 'icon' | 'sm' | 'md' | 'lg';
		onclick?: (e: Event) => unknown;
	} = $props();

	const button = tv({
		base: 'flex gap-2 select-none cursor-pointer place-content-center place-items-center font-semibold text-white rounded-xl active:opacity-80',
		variants: {
			color: {
				primary: 'bg-blue-500 text-white shadow-lg shadow-blue-500/60',
				secondary: 'bg-slate-500 text-white shadow-lg shadow-slate-500/60',
				outline: 'text-slate-600 border-slate-500 border-2',
				destructive: 'bg-red-500 text-white'
			},
			size: {
				icon: 'px-6 py-2',
				sm: 'text-sm px-6 py-2',
				md: 'text-base px-8 py-2',
				lg: 'px-12 py-3 text-lg'
			}
		},
		defaultVariants: {
			size: 'md',
			color: 'primary'
		}
	});
</script>

<a class="contents" {href}>
	<button {...attr} class={twMerge(button({ color: variant, size }), className)}>
		{@render children()}
	</button>
</a>
