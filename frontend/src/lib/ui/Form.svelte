<script lang="ts">
	import type { Snippet } from 'svelte';

	const {
		children,
		className,
		onSubmit = async () => {}
	}: {
		children: Snippet;
		className?: string;
		onSubmit?: (formData: FormData, ev: SubmitEvent) => Promise<unknown>;
	} = $props();

	const handleSubmit = async (ev: SubmitEvent) => {
		ev.preventDefault();
		const formData = new FormData(ev.target as HTMLFormElement);
		await onSubmit(formData, ev).catch(console.log);
	};
</script>

<form class="flex flex-col gap-2 {className}" onsubmit={handleSubmit}>
	{@render children()}
</form>
