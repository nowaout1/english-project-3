import { redirect } from '@sveltejs/kit';

import { api } from '$lib/api';
import { createRoom } from '$lib/state/index.svelte';

export const load = async ({ params: { id } }) => {
	if (id) {
		const { error } = await api.joinRoom(id);

		if (error == null) {
			return {
				room: createRoom(id)
			};
		}
	}

	redirect(307, '/');
};
