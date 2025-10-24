<script lang="ts">
	import { goto } from '$app/navigation';

	import Title from '$lib/ui/Title.svelte';
	import Delimiter from '$lib/ui/Delimiter.svelte';
	import EnterRoom from '$lib/widgets/EnterRoom.svelte';
	import CreateRoom from '$lib/widgets/CreateRoom.svelte';

	const onRoomEnter = ({ roomId }: { roomId: string }) => {
		goto(`/room/${roomId}`);
	};

	const onRoomCreated = ({ roomId }: { roomId: string }) => {
		goto(`/room/${roomId}`);
	};
</script>

<section class="flex min-h-dvh flex-col items-center justify-end gap-6 p-20">
	<Title>Math battle!</Title>
	<EnterRoom className="max-w-120 w-[80vw]" {onRoomEnter} />
	<Delimiter>or</Delimiter>
	<CreateRoom {onRoomCreated} />
</section>
