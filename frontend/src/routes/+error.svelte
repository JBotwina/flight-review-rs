<script lang="ts">
	import { page } from '$app/state';

	let status = $derived(page.status);
	let message = $derived(page.error?.message ?? '');

	let isNotFound = $derived(status === 404);

	let heading = $derived(isNotFound ? 'Page not found' : 'Something went wrong');
	let blurb = $derived(
		isNotFound
			? "Sorry, we couldn't find the page you're looking for. The log may have been removed, or the link is incorrect."
			: 'An unexpected error occurred while loading this page. Try again, or head back to a known-good page.'
	);
</script>

<svelte:head>
	<title>{status} · Flight Review</title>
</svelte:head>

<div class="grid min-h-[calc(100vh-4rem)] place-items-center bg-white px-6 py-24 sm:py-32 lg:min-h-screen lg:px-8 dark:bg-gray-900">
	<div class="text-center">
		<p class="text-base font-semibold text-indigo-600 dark:text-indigo-400">{status}</p>
		<h1 class="mt-4 text-balance text-5xl font-semibold tracking-tight text-gray-900 sm:text-7xl dark:text-gray-100">
			{heading}
		</h1>
		<p class="mt-6 text-pretty text-lg font-medium text-gray-500 sm:text-xl/8 dark:text-gray-400">
			{blurb}
		</p>
		{#if message && message !== heading}
			<p class="mt-3 font-mono text-sm text-gray-400 dark:text-gray-500">{message}</p>
		{/if}
		<div class="mt-10 flex items-center justify-center gap-x-6">
			<a
				href="/"
				class="rounded-md bg-indigo-600 px-3.5 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
			>
				Go home
			</a>
			<a
				href="/browse"
				class="text-sm font-semibold text-gray-900 dark:text-gray-100"
			>
				Browse logs <span aria-hidden="true">&rarr;</span>
			</a>
		</div>
	</div>
</div>
