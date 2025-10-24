<script lang="ts">
	import { onMount } from 'svelte';

	import '../app.css';
	import { LoadBar, loadbar } from '$lib/ui/load-bar';
	import { api } from '$lib/api';
	import { user } from '$lib/state/index.svelte';

	const { children } = $props();

	onMount(() => {
		// Update user when connected
		const unsubscribeOnConnected = api.onConnected(async () => {
			const { data } = await api.user();

			if (data) {
				user.update({ id: data.user_id, username: data.username });
			}
		});

		// Show / hide loadbar when requesting
		const unsubscribeRequestStart = api.onRequestStart(loadbar.show);
		const unsubscribeRequestEnd = api.onRequestEnd(loadbar.hide);

		return () => {
			api.disconnect();

			unsubscribeOnConnected();
			unsubscribeRequestStart();
			unsubscribeRequestEnd();
		};
	});
</script>

<svelte:head>
	<title>Math Battle</title>
	<meta name="description" content="Math battle web application" />
</svelte:head>

<div class="app">
	<main>
		<LoadBar />
		{@render children()}
	</main>
</div>
