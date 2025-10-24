<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Trophy, X, Timer } from '@lucide/svelte';

	import Button from '$lib/ui/Button.svelte';
	import { api } from '$lib/api';
	import { createLeaderboard, createQuiz } from '$lib/state';

	const { data } = $props();

	onMount(() => {
		api.questionQuiz();
		api.timerQuiz();

		const unsubscribeQuestion = api.onMessage('quiz_question', ({ data }) => {
			quiz.resetQuestionData();

			if (data) {
				quiz.setExpression(data.expression);
				quiz.setAnswerVariants(
					data.variants.map((x) => ({
						variantId: x.variant_id,
						questionId: x.question_id,
						value: x.value
					}))
				);
			}
		});

		const unsubscribeCheckAnswer = api.onMessage('quiz_check', ({ data }) => {
			if (data) {
				quiz.setIsCorrect(data.is_correct);
			}
		});

		const unsubscribeTimer = api.onMessage('quiz_timer', ({ data }) => {
			if (data) {
				quiz.setCountdown(data.remaining_secs);
			}
		});

		const unsubscribeLeaderboard = api.onMessage('quiz_leaderboard', ({ data }) => {
			if (data) {
				leaderboard.updateLeaderboard(
					data.participants.map((p) => ({
						id: p.id,
						username: p.username,
						score: p.score
					}))
				);
			}
		});

		const unsubscribeFinished = api.onMessage('quiz_finished', () => {
			view = 'leaderboard';
		});

		return () => {
			api.leaveQuiz();
			unsubscribeQuestion();
			unsubscribeCheckAnswer();
			unsubscribeTimer();
			unsubscribeLeaderboard();
			unsubscribeFinished();
		};
	});

	const goToRoom = () => goto(`/room/${data.room.roomId}`);

	const quiz = createQuiz();
	const leaderboard = createLeaderboard();

	const handleCompare = async () => {
		if (quiz.selectedVariant) {
			api.questionCheckAnswer(quiz.selectedVariant);
		}
	};

	let view = $state<'quiz' | 'leaderboard'>('quiz');

	const switchView = () => {
		if (view === 'quiz') {
			view = 'leaderboard';
		} else {
			view = 'quiz';
		}
	};
</script>

<section class="flex h-svh flex-col">
	<header class="flex min-h-[20svh] w-full items-center justify-between gap-4 px-6">
		<Button size="icon" variant="destructive" onclick={goToRoom}>
			<X />
		</Button>
		<div class="flex gap-2">
			<span
				class="flex min-w-20 items-center justify-center gap-2 rounded-full bg-linear-to-b from-sky-400 to-sky-600 px-4 py-2 text-2xl font-semibold text-white select-none"
			>
				{quiz.countdown}
			</span>
			<Button variant="secondary" size="sm" className="rounded-full" onclick={switchView}>
				<Trophy />
			</Button>
		</div>
	</header>
	<div class="flex h-[80svh] w-full flex-col place-items-center gap-12 p-6">
		{#if view === 'leaderboard'}
			<strong class="text-3xl font-semibold">Leaderboard</strong>
			<div
				class="flex w-full flex-col gap-2 overflow-y-auto rounded-2xl border border-slate-300 p-2"
			>
				{#each leaderboard.participants as participant, idx (participant.id)}
					<div class="flex w-full items-center justify-between gap-2 rounded-lg bg-white px-4 py-2">
						<span class="w-3/4 truncate text-lg font-medium">
							{idx + 1}. {participant.username}
						</span>
						<span class="text-lg font-semibold">{participant.score}</span>
					</div>
				{/each}
			</div>
		{:else}
			<div
				class="flex w-full place-content-center rounded-xl bg-white p-4 text-2xl font-semibold text-slate-600 select-none"
			>
				{quiz.expression}
			</div>
			<div class="flex size-full flex-col place-items-center gap-4">
				<span class="text-2xl">Choose the correct answer</span>
				<div class="grid size-full grid-cols-2 gap-2">
					{#each quiz.answerVariants as variant, idx (idx)}
						{@const isSelected = variant.variantId === quiz.selectedVariant?.variantId}
						{@const isSelectedStyles = `border-4 ${isSelected ? 'border-blue-500' : 'border-transparent'}`}
						{@const isCorrectStyles = isSelected && quiz.isCorrect ? 'border-green-500' : ''}
						{@const isIncorrectStyles = isSelected && quiz.isIncorrect ? 'border-orange-500' : ''}
						<Button
							className="relative overflow-hidden bg-white select-none text-3xl text-slate-600 {isSelectedStyles} {isCorrectStyles} {isIncorrectStyles}"
							size="lg"
							variant="secondary"
							onclick={() => quiz.setSelectedVariant(variant)}
							disabled={quiz.isAnswered}
						>
							{variant.value}
						</Button>
					{/each}
				</div>
			</div>
			<form class="flex w-full place-content-center">
				<Button
					className="w-full"
					size="lg"
					disabled={quiz.isCountdownFinished}
					onclick={handleCompare}
				>
					Compare
				</Button>
			</form>
		{/if}
	</div>
</section>
