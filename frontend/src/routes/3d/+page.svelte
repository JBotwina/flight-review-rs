<script lang="ts">
	import { page } from '$app/state';
	const logId = $derived(page.url.searchParams.get('log') ?? page.url.searchParams.get('id') ?? '');
	const mapHref = $derived(logId ? `/log/${encodeURIComponent(logId)}/map` : '/browse');
</script>

<svelte:head>
	<title>Legacy 3D replay compatibility</title>
</svelte:head>

<section class="mx-auto max-w-3xl px-4 py-12">
	<div class="rounded-lg bg-white p-8 shadow-sm ring-1 ring-gray-200">
		<p class="text-sm font-semibold uppercase tracking-wide text-indigo-600">Legacy route compatibility</p>
		<h1 class="mt-2 text-3xl font-bold tracking-tight text-gray-900">3D flight replay moved in Flight Review v2</h1>
		<p class="mt-4 text-gray-600">
			The legacy Python app exposed <code>/3d</code> for Cesium replay links. In v2, map and replay context lives with the log detail experience instead of a standalone legacy route.
		</p>
		{#if logId}
			<p class="mt-4 text-gray-600">Log ID from the legacy URL: <code>{logId}</code></p>
			<a class="mt-6 inline-flex rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-500" href={mapHref}>Open this log map</a>
		{:else}
			<a class="mt-6 inline-flex rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-500" href="/browse">Browse logs</a>
		{/if}
	</div>
</section>
