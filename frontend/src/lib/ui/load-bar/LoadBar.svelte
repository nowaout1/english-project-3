<script module lang="ts">
	let isVisible = $state(false);

	export const loadbar = {
		show: () => (isVisible = true),
		hide: () => (isVisible = false)
	};
</script>

<script lang="ts">
	import { fade } from 'svelte/transition';
	import Spinner from './Spinner.svelte';
</script>

{#if isVisible}
	<button
		onclick={() => (isVisible = false)}
		class="fixed top-20 right-1/2 left-1/2 z-20 flex w-fit -translate-1/2 place-content-center rounded-2xl bg-slate-300/40 px-6 py-2 backdrop-blur-lg"
		transition:fade
	>
		<Spinner />
	</button>
{/if}
