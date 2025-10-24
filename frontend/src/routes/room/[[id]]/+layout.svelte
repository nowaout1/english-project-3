<script>
	import { onMount } from 'svelte';

	import { api } from '$lib/api';

	const { children, data } = $props();
	const { room } = data;

	onMount(() => {
		api.requestRoomMembersList().then(({ data }) => {
			if (data?.members) {
				room.addMembers(...data?.members);
			}
		});
	});
</script>

{@render children()}
