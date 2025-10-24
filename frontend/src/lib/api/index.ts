import type { Result } from '$lib/types';

const WS_ADDR = import.meta.env.VITE_WS_ADDR;
const WS_CONN_TIMEOUT_MS = import.meta.env.VITE_WS_CONN_TIMEOUT_MS;
const WS_RESPONSE_TIMEOUT_MS = import.meta.env.VITE_WS_RESPONSE_TIMEOUT_MS;
const WS_RECONNECT_ATTEMPTS = import.meta.env.VITE_WS_RECONNECT_ATTEMPTS;

export type Request =
	| User
	| UsernameUpdated
	| RoomCreate
	| RoomJoin
	| RoomLeave
	| RoomMemberJoined
	| RoomMemberLeaved
	| RoomMembersList
	| QuizStart
	| QuizQuestion
	| QuizCheckAnswer
	| QuizTimer
	| QuizLeaderboard
	| QuizLeave
	| QuizFinished;

export type User = {
	send: Message<'user', void>;
	read: Message<'user', { user_id: string; username: string }>;
};

export type UsernameUpdated = {
	send: Message<'username_updated', { username: string }>;
	read: Message<'username_updated', { user_id: string; username: string }, 'INVALID_USERNAME'>;
};

export type RoomCreate = {
	send: Message<'room_create', void>;
	read: Message<'room_create', { room_id: string }>;
};

export type RoomJoin = {
	send: Message<'room_join', { room_id: string }>;
	read: Message<'room_join', void, 'INVALID_ID' | 'NOT_FOUND' | 'CROWDED'>;
};

export type RoomLeave = {
	send: Message<'room_leave', void>;
	read: Message<'room_leave', void, 'INVALID_ID' | 'NOT_FOUND'>;
};

export type RoomMemberJoined = {
	send: Message<'room_member_joined', void>;
	read: Message<'room_member_joined', { member: UserDto }>;
};

export type RoomMemberLeaved = {
	send: Message<'room_member_leaved', void>;
	read: Message<'room_member_leaved', { member_id: UserDto['id'] }>;
};

export type RoomMembersList = {
	send: Message<'room_members_list', void>;
	read: Message<'room_members_list', { members: UserDto[] }, 'NOT_FOUND'>;
};

export type QuizStart = {
	send: Message<'quiz_start', void>;
	read: Message<'quiz_start', { quiz_id: string }, 'NOT_FOUND'>;
};

export type QuizQuestion = {
	send: Message<'quiz_question', void>;
	read: Message<'quiz_question', { id: string; expression: string; variants: AnswerVariantDto[] }>;
};

export type QuizCheckAnswer = {
	send: Message<'quiz_check', { question_id: string; variant_id: string }>;
	read: Message<
		'quiz_check',
		{
			user_id: string;
			is_correct: boolean;
			submitted_variant_id: string;
			correct_variant_id: string;
		},
		'INVALID_ID' | 'NOT_FOUND' | 'NOT_RELEVANT' | 'ALREADY_EXISTS'
	>;
};

export type QuizTimer = {
	send: Message<'quiz_timer', void>;
	read: Message<'quiz_timer', { remaining_secs: number }>;
};

export type QuizLeaderboard = {
	send: Message<'quiz_leaderboard', void>;
	read: Message<'quiz_leaderboard', { participants: ParticipantDto[] }, 'INVALID_ID' | 'NOT_FOUND'>;
};

export type QuizLeave = {
	send: Message<'quiz_leave', void>;
	read: Message<'quiz_leave', void, 'INVALID_ID' | 'NOT_FOUND'>;
};

export type QuizFinished = {
	send: Message<'quiz_finished', void>;
	read: Message<'quiz_finished', void>;
};

type Message<
	Kind extends MessageKind,
	Data,
	Code extends MessageErrorCode | undefined = undefined
> = {
	kind: Kind;
	data?: Data;
	error?: { code: Code };
};

type MessageKind =
	| 'user'
	| 'username_updated'
	| 'room_create'
	| 'room_join'
	| 'room_leave'
	| 'room_member_joined'
	| 'room_member_leaved'
	| 'room_members_list'
	| 'quiz_start'
	| 'quiz_question'
	| 'quiz_check'
	| 'quiz_timer'
	| 'quiz_score'
	| 'quiz_leaderboard'
	| 'quiz_leave'
	| 'quiz_finished';

type MessageErrorCode =
	| 'INVALID_ID'
	| 'INVALID_USERNAME'
	| 'NOT_FOUND'
	| 'CROWDED'
	| 'NOT_RELEVANT'
	| 'ALREADY_EXISTS';

export type UserDto = { id: string; username: string };

export type ParticipantDto = { id: string; username: string; score: number };

export type AnswerVariantDto = {
	variant_id: string;
	question_id: string;
	value: string;
};

class Api {
	constructor(
		public readonly addr: string = WS_ADDR,
		public readonly responseTimeoutMs: number = WS_RESPONSE_TIMEOUT_MS,
		private readonly connection = new Connection(addr),
		private readonly listeners = {
			start: new Set<Subscriber>(),
			end: new Set<Subscriber>()
		} as const
	) {}

	public get isConnected(): boolean {
		return this.connection.isOpen;
	}

	public get isDisconnected(): boolean {
		return this.connection.isClosed;
	}

	public async connect() {
		await this.connection.connect();
	}

	public async disconnect() {
		await this.connection.disconnect();
	}

	public onConnected(fn: Subscriber): Unsubscribe {
		return this.connection.on('open', fn);
	}

	public onDisconnected(fn: Subscriber): Unsubscribe {
		return this.connection.on('close', fn);
	}

	public onRequestStart(fn: Subscriber): Unsubscribe {
		this.listeners['start'].add(fn);
		return () => this.listeners['start'].delete(fn);
	}

	public onRequestEnd(fn: Subscriber): Unsubscribe {
		this.listeners['end'].add(fn);
		return () => this.listeners['end'].delete(fn);
	}

	public onMessage<K extends Request['read']['kind']>(
		kind: K,
		subscriber: Subscriber<[Extract<Request['read'], { kind: K }>]>
	): Unsubscribe {
		const unsubscribe = this.connection.on('msg', (msg: Extract<Request['read'], { kind: K }>) => {
			if (msg && msg.kind === kind) {
				subscriber(msg);
			}
		});

		return unsubscribe;
	}

	public async user(): Promise<User['read']> {
		return this.request({ kind: 'user' });
	}

	public async updateUsername(username: string): Promise<UsernameUpdated['read']> {
		return this.request({ kind: 'username_updated', data: { username } });
	}

	public async createRoom(): Promise<RoomCreate['read']> {
		return this.request({ kind: 'room_create' });
	}

	public async joinRoom(roomId: string): Promise<RoomJoin['read']> {
		return this.request({ kind: 'room_join', data: { room_id: roomId } });
	}

	public async leaveRoom(): Promise<RoomLeave['read']> {
		return this.request({ kind: 'room_leave' });
	}

	public async requestRoomMembersList(): Promise<RoomMembersList['read']> {
		return this.request({ kind: 'room_members_list' });
	}

	public async startQuiz(): Promise<QuizStart['read']> {
		return this.request({ kind: 'quiz_start' });
	}

	public async questionQuiz(): Promise<QuizQuestion['read']> {
		return this.request({ kind: 'quiz_question' });
	}

	public async questionCheckAnswer({
		questionId,
		variantId
	}: {
		questionId: string;
		variantId: string;
	}): Promise<QuizCheckAnswer['read']> {
		return this.request({
			kind: 'quiz_check',
			data: { question_id: questionId, variant_id: variantId }
		});
	}

	public async timerQuiz(): Promise<QuizQuestion['read']> {
		return this.request({ kind: 'quiz_timer' });
	}

	public async leaderboardQuiz(): Promise<QuizLeaderboard['read']> {
		return this.request({ kind: 'quiz_leaderboard' });
	}

	public async leaveQuiz(): Promise<QuizLeave['read']> {
		return this.request({ kind: 'quiz_leave' });
	}

	private async request<RS extends Request['send']>(
		msg: RS
	): Promise<Extract<Request, RS>['read']> {
		this.emit('start');

		const request = new Promise((resolve) => {
			const timeoutId = setTimeout(
				() => resolve({ kind: msg.kind, error: { msg: 'Timeout exceeded' } }),
				this.responseTimeoutMs
			);

			const unsubscribe = this.onMessage(msg.kind, (msg) => {
				clearTimeout(timeoutId);
				unsubscribe();
				resolve(msg);
			});

			this.send(msg);
		});

		request.then(() => this.emit('end'));

		return request as never;
	}

	private async send(msg: Request['send']): Promise<Result<void, unknown>> {
		return this.connection.send(msg);
	}

	private emit(event: keyof Api['listeners']) {
		this.listeners[event].forEach((fn) => fn());
	}
}

type ConnectionEvent = keyof Connection['subscribers'];
type Subscriber<T extends unknown[] = []> = (...args: T) => unknown;
type Unsubscribe = () => unknown;

class Connection {
	constructor(
		public readonly addr: string,
		public readonly reconnectAttempts: number = WS_RECONNECT_ATTEMPTS,
		public readonly connectionTimeoutMs: number = WS_CONN_TIMEOUT_MS,
		private ws: WebSocket | null = null,
		private readonly subscribers = {
			open: new Set<Subscriber>(),
			close: new Set<Subscriber>(),
			msg: new Set<Subscriber<[unknown]>>()
		} as const
	) {}

	public on<T extends unknown[]>(event: ConnectionEvent, fn: Subscriber<T>): Unsubscribe {
		this.subscribers[event].add(fn);
		return () => this.subscribers[event].delete(fn);
	}

	public async send(msg: object): Promise<Result<void, unknown>> {
		try {
			const ws = await this.getConnection();
			const serialized = JSON.stringify(msg);
			console.debug(`Send: ${serialized}`);
			ws.send(serialized);
			return { error: null };
		} catch (error) {
			return { error };
		}
	}

	public async connect() {
		if (this.isOpen) {
			console.log(`Already connection was established with ${this.addr}`);
			return;
		}

		if (this.isConnecting) {
			console.log(`Already trying to connect to ${this.addr}`);
			return;
		}

		for (let i = 1; i <= this.reconnectAttempts && this.ws == null; i++) {
			console.log(`Trying connect to ${this.addr} (attempt: ${i})`);

			await this.tryEstablish();

			if (this.ws) {
				console.log(`Connection was sucessfully established with ${this.addr}`);
				this.eventEmitter(this.ws);
				break;
			} else {
				console.log(`Connection failed with ${this.addr}`);
			}
		}
	}

	public disconnect() {
		this.ws?.close();
		this.ws = null;
		this.emit('close');
	}

	private async tryEstablish() {
		this.disconnect();
		this.ws = new WebSocket(this.addr);

		const listen = async (ws: WebSocket, timeoutMs: number) => {
			return new Promise<WebSocket | null>((resolve) => {
				const timeoutId = setTimeout(() => {
					resolve(null);
					ws.close();
				}, timeoutMs);

				const clear = () => {
					clearTimeout(timeoutId);
					ws.onopen = null;
					ws.onerror = null;
					ws.onclose = null;
				};

				ws.onopen = () => {
					clear();
					resolve(ws);
					this.emit('open');
				};
				ws.onclose = () => resolve(null);
				ws.onerror = () => resolve(null);
			});
		};

		this.ws = await listen(this.ws, this.connectionTimeoutMs);
	}

	private eventEmitter(ws: WebSocket) {
		ws.onmessage = ({ data }) => {
			console.debug(`Recv: ${data}`);
			const parseJson = (rawJson: string) => {
				try {
					const result = JSON.parse(rawJson);
					if (typeof result?.data === 'string') result.data = JSON.parse(result.data);
					if (typeof result?.error === 'string') result.error = JSON.parse(result.error);
					return result;
				} catch (error) {
					console.debug(`Failed to parse message: ${error}`);
					return null;
				}
			};
			this.emit('msg', parseJson(data));
		};
		ws.onerror = () => {
			console.log(`Connection was interrupted  with ${this.addr}`);
			this.ws = null;
			this.emit('close');
		};
		ws.onclose = () => {
			console.log(`Connection was closed with ${this.addr}`);
			this.ws = null;
			this.emit('close');
		};
	}

	private emit(event: ConnectionEvent, data?: unknown) {
		this.subscribers[event].forEach((fn) => fn(data));
	}

	public get isOpen(): boolean {
		return this.ws != null && this.ws.readyState === WebSocket.OPEN;
	}

	public get isConnecting(): boolean {
		return this.ws != null && this.ws.readyState === WebSocket.CONNECTING;
	}

	public get isClosed(): boolean {
		return (
			this.ws == null ||
			this.ws.readyState === WebSocket.CLOSING ||
			this.ws.readyState === WebSocket.CLOSED
		);
	}

	// PANICS: if connection was not established
	private async getConnection(): Promise<WebSocket> {
		if (this.isClosed) {
			await this.connect();
		}

		if (this.ws && this.isOpen) {
			return this.ws;
		}

		this.ws = null;

		throw new Error("Can't establish connection");
	}
}

export const api = new Api();
