<svelte:head>
	<title>Legacy radio controller compatibility</title>
</svelte:head>

<section class="mx-auto max-w-3xl px-4 py-12">
	<div class="rounded-lg bg-white p-8 shadow-sm ring-1 ring-gray-200">
		<p class="text-sm font-semibold uppercase tracking-wide text-indigo-600">Legacy route compatibility</p>
		<h1 class="mt-2 text-3xl font-bold tracking-tight text-gray-900">Radio controller visualizer</h1>
		<p class="mt-4 text-gray-600">
			The legacy Python Flight Review app exposed <code>/radio_controller</code> as a testing-only radio controller visualization harness. Flight Review v2 does not yet have a standalone RC visualizer, so this compatibility page makes the route explicit instead of returning a 404.
		</p>
		<div class="mt-6 flex flex-wrap gap-3">
			<a class="rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-500" href="/">Upload a log</a>
			<a class="rounded-md bg-white px-4 py-2 text-sm font-semibold text-gray-900 ring-1 ring-gray-300 hover:bg-gray-50" href="/browse">Browse logs</a>
		</div>
	</div>
</section>
