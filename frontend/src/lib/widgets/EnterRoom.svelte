<script lang="ts">
	import Button from '$lib/ui/Button.svelte';
	import Form from '$lib/ui/Form.svelte';
	import Input from '$lib/ui/Input.svelte';

	const {
		onRoomEnter,
		className
	}: {
		onRoomEnter: (data: { roomId: string }) => void;
		className?: string;
	} = $props();

	const onSubmit = async (form: FormData) => {
		const roomId = form.get('room-id') as string | null;

		if (roomId) {
			onRoomEnter({ roomId });
		}
	};
</script>

<Form {className} {onSubmit}>
	<Input name="room-id" placeholder="Enter room ID" size="lg" />
	<Button size="lg">Go in</Button>
</Form>
