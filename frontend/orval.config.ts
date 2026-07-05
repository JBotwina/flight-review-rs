import { defineConfig } from 'orval';

export default defineConfig({
	flightReview: {
		input: '../openapi/openapi.json',
		output: {
			mode: 'tags-split',
			target: './src/lib/generated/endpoints',
			schemas: './src/lib/generated/models',
			client: 'svelte-query',
			baseUrl: '',
			override: {
				mutator: {
					path: './src/lib/generated/custom-fetch.ts',
					name: 'customFetch',
				},
			},
		},
	},
});
