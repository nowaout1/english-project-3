<script lang="ts">
	import { api } from '$lib/api';
	import Button from '$lib/ui/Button.svelte';
	import Form from '$lib/ui/Form.svelte';

	const {
		onRoomCreated,
		className
	}: {
		onRoomCreated: (data: { roomId: string }) => void;
		className?: string;
	} = $props();

	const onSubmit = async () => {
		const { data, error } = await api.createRoom();

		if (data) {
			onRoomCreated({ roomId: data.room_id });
		}
	};
</script>

<Form {className} {onSubmit}>
	<Button variant="secondary" size="lg">Create room</Button>
</Form>
