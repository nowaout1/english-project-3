<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	import { api } from '$lib/api';
	import { user } from '$lib/state/index.svelte.js';
	import Button from '$lib/ui/Button.svelte';
	import Delimiter from '$lib/ui/Delimiter.svelte';
	import Input from '$lib/ui/Input.svelte';
	import { Sword, X } from '@lucide/svelte';

	const { data } = $props();
	const { room } = data;

	const leaveRoom = async () => {
		goto('/');
		await api.leaveRoom();
	};

	onMount(() => {
		const unsubscribeQuizStarted = api.onMessage('quiz_start', ({ data }) => {
			if (data) {
				goto(`/room/${room.roomId}/quiz`);
			}
		});

		const unsubscribeUsernameUpdated = api.onMessage('username_updated', ({ data }) => {
			if (data) {
				room.updateUsername(data.user_id, data.username);
			}
		});

		const unsubscribeMemberJoined = api.onMessage('room_member_joined', ({ data }) => {
			if (data?.member) {
				room.addMembers(data?.member);
			}
		});

		const unsubscribeMemberLeaved = api.onMessage('room_member_leaved', ({ data }) => {
			if (data?.member_id) {
				room.removeMember(data?.member_id);
			}
		});

		return () => {
			unsubscribeQuizStarted();
			unsubscribeUsernameUpdated();
			unsubscribeMemberJoined();
			unsubscribeMemberLeaved();
		};
	});

	let username = $state(user.username);

	$effect(() => {
		api.updateUsername(username).then(({ data }) => {
			if (data) {
				user.setUsername(data.username);
			}
		});
	});

	const startQuiz = async () => {
		const { error } = await api.startQuiz();

		if (error?.code === 'NOT_FOUND') {
			goto('/');
		}
	};
</script>

<section class="flex h-svh flex-col">
	<header class="flex h-[20svh] w-full items-center justify-between gap-4 px-6">
		<Button size="icon" variant="destructive" onclick={leaveRoom}>
			<X />
		</Button>
	</header>
	<div class="flex h-[80svh] flex-col justify-end gap-4 p-6">
		<form class="flex w-full flex-col items-center gap-1">
			<label for="room-id" class="px-4 text-xl font-semibold">Room ID</label>
			<Input
				id="room-id"
				className="w-full text-center select-all"
				size="lg"
				readonly
				value={room.roomId}
				copyOnClick={true}
			/>
		</form>

		<form class="flex w-full flex-col items-center gap-1">
			<label for="room-id" class="px-4 text-xl font-semibold">Your username</label>
			<Input
				id="username"
				className="w-full text-center"
				size="lg"
				placeholder="Enter username"
				bind:value={username}
			/>
		</form>

		<div class="flex h-[23vh] flex-col items-center gap-1 rounded-lg border-2 border-slate-400 p-2">
			<Delimiter>
				<strong>Members</strong>
			</Delimiter>
			<div class="flex w-full flex-col overflow-auto">
				{#each room.members as { id, username }, idx (id)}
					<div class="flex w-full p-2 font-semibold">{idx + 1}. {username}</div>
				{/each}
			</div>
		</div>

		<!-- TODO: quiz options -->
		<!-- <div class="grid grid-cols-2 grid-rows-2 gap-2">
			<Button
				className="bg-linear-to-r from-orange-600 to-orange-400 rounded-3xl aspect-square p-4 text-lg justify-start"
				color="secondary">Complexity</Button
			>
			<Button
				className="bg-linear-to-r from-green-600 to-green-400 rounded-3xl aspect-square p-4 text-lg justify-start"
				color="secondary">Members</Button
			>
			<Button
				className="bg-linear-to-r from-pink-600 to-pink-400 rounded-3xl aspect-square p-4 text-lg justify-start"
				color="secondary">Interval</Button
			>
			<Button
				className="bg-linear-to-r from-indigo-600 to-indigo-400 rounded-3xl aspect-square p-4 text-lg justify-start"
				color="secondary">Operations</Button
			>
		</div> -->

		<Button size="lg" onclick={startQuiz}>
			<Sword />
			Start battle
		</Button>
	</div>
</section>
