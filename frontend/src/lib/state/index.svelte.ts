import { SvelteMap } from 'svelte/reactivity';

export type User = { id: string; username: string };
export type Participant = { id: string; username: string; score: number };

export const createRoom = (roomId: string) => {
	const roomMembers = new SvelteMap<User['id'], User>();

	return {
		get roomId() {
			return roomId;
		},
		get members() {
			return roomMembers.values();
		},
		addMembers(...members: User[]) {
			for (const m of members) {
				roomMembers.set(m.id, m);
			}
		},
		removeMember(memberId: User['id']) {
			roomMembers.delete(memberId);
		},
		updateUsername(memberId: User['id'], username: User['username']) {
			const member = roomMembers.get(memberId);

			if (member) {
				roomMembers.set(memberId, { ...member, username });
			}
		}
	};
};

export const createUser = () => {
	let id = $state<string | null>(null);
	let username = $state<string>('');

	return {
		get userId() {
			return id;
		},
		get username() {
			return username;
		},
		set username(newUsername) {
			username = newUsername;
		},
		update(data: Partial<User>) {
			if (data?.id) id = data.id;
			if (data?.username) username = data.username;
		},
		setId(newId: string) {
			id = newId;
			return this;
		},
		setUsername(newUsername: string) {
			username = newUsername;
			return this;
		}
	};
};

export type AnswerVariant = {
	variantId: string;
	questionId: string;
	value: string;
};

export const createQuiz = () => {
	const resetQuestionData = () => {
		expression = null;
		answerVariants = [];
		selectedVariant = null;
		isCorrect = null;
		countdown = 0;
	};

	let expression = $state<string | null>(null);
	let answerVariants = $state<AnswerVariant[]>([]);
	let selectedVariant = $state<AnswerVariant | null>(null);
	let isCorrect = $state<boolean | null>(null);
	let countdown = $state<number>(0);

	const isCountdownFinished = $derived<boolean>(countdown === 0);
	const isIncorrect = $derived<boolean | null>(isCorrect == null ? null : !isCorrect);
	const isAnswered = $derived<boolean>(isCountdownFinished || isCorrect != null);

	return {
		resetQuestionData,
		get expression() {
			return expression;
		},
		setExpression(newExpression: string) {
			expression = newExpression;
		},
		get answerVariants() {
			return answerVariants;
		},
		setAnswerVariants(newAnswerVariants: AnswerVariant[]) {
			answerVariants = newAnswerVariants;
		},
		get selectedVariant() {
			return selectedVariant;
		},
		setSelectedVariant(variant: AnswerVariant) {
			selectedVariant = variant;
		},
		get isCorrect() {
			return isCorrect;
		},
		setIsCorrect(newIsCorrect: boolean) {
			isCorrect = newIsCorrect;
		},
		get countdown() {
			return countdown;
		},
		setCountdown(newCountdown: number) {
			countdown = newCountdown;
		},
		get isCountdownFinished() {
			return isCountdownFinished;
		},
		get isIncorrect() {
			return isIncorrect;
		},
		get isAnswered() {
			return isAnswered;
		}
	};
};

export const createLeaderboard = () => {
	let participants = $state<Participant[]>([]);

	return {
		get participants() {
			return participants;
		},
		updateLeaderboard(newParticipants: Participant[]) {
			participants = newParticipants;
		}
	};
};

export const user = createUser();
