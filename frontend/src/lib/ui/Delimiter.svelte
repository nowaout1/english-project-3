<script lang="ts">
	const { children } = $props();
</script>

<div class="flex w-3/4 items-center justify-center gap-2 text-slate-400">
	<hr class="w-full border" />
	<span>{@render children()}</span>
	<hr class="w-full border" />
</div>
