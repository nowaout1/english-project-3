import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	esbuild: {
		pure: process.env.NODE_ENV === 'production' ? ['console.debug'] : []
	}
});
