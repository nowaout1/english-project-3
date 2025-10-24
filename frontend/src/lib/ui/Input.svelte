<script module lang="ts">
	const copyText = async (text: string) => {
		try {
			await navigator.clipboard.writeText(text);
		} catch (err) {
			console.error(err);
		}
	};
</script>

<script lang="ts">
	import { tv } from 'tailwind-variants';
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { twMerge } from 'tailwind-merge';

	let {
		value = $bindable(''),
		copyOnClick = false,
		className,
		color,
		size,
		...attr
	}: {
		className?: string;
		copyOnClick?: boolean;
		color?: 'primary';
		size?: 'md' | 'lg';
	} & Omit<HTMLInputAttributes, 'size'> = $props();

	const input = tv({
		base: 'placeholder-gray-500 rounded-xl border-slate-400 outline-slate-400',
		variants: {
			color: {
				primary: '',
				secondary: ''
			},
			size: {
				sm: 'text-sm border-2',
				md: 'text-md px-3 py-2 border-3',
				lg: 'text-lg px-4 py-3 text-lg border-4'
			}
		},
		defaultVariants: {
			size: 'md',
			color: 'primary'
		}
	});

	const handleClick = (ev: Event) => {
		const target = ev.target as HTMLInputElement;

		if (copyOnClick) {
			copyText(target.value);
		}
	};
</script>

<input
	class={twMerge(input({ color, size }), className)}
	bind:value
	{...attr}
	onclick={handleClick}
/>
