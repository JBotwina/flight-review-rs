<script lang="ts">
	import { createVersion } from '$lib/generated/endpoints/version/version';

	// Frontend version is baked in at build time (Vite define). Backend versions
	// are fetched at runtime; they can differ since the two deploy separately.
	const frontend = {
		version: __APP_VERSION__,
		git_sha: __GIT_SHA__,
		build_time: __BUILD_TIME__,
	};

	const versionQuery = createVersion({ query: { staleTime: 5 * 60_000, retry: 0 } });
	let backend = $derived($versionQuery.data ?? null);
	let backendFailed = $derived($versionQuery.isError);

	function shortDate(iso: string): string {
		if (!iso || iso === 'unknown') return 'unknown';
		// Trim "2026-06-25T16:21:14Z" → "2026-06-25" for the compact rows.
		return iso.slice(0, 10);
	}
</script>

<details class="group text-xs text-gray-400 dark:text-gray-500">
	<summary class="cursor-pointer list-none font-medium hover:text-gray-600 dark:hover:text-gray-300">
		v{frontend.version}
		<span class="opacity-60 group-open:hidden">▸</span>
		<span class="opacity-60 hidden group-open:inline">▾</span>
	</summary>
	<dl class="mt-1.5 space-y-1">
		<div>
			<dt class="font-medium text-gray-500 dark:text-gray-400">Frontend</dt>
			<dd class="tabular-nums">
				{frontend.version} · {frontend.git_sha} · built {shortDate(frontend.build_time)}
			</dd>
		</div>
		{#if backend}
			<div>
				<dt class="font-medium text-gray-500 dark:text-gray-400">Server</dt>
				<dd class="tabular-nums">
					{backend.server} · {backend.git_sha} · built {shortDate(backend.build_time)}
				</dd>
			</div>
			<div>
				<dt class="font-medium text-gray-500 dark:text-gray-400">Converter</dt>
				<dd class="tabular-nums">{backend.converter}</dd>
			</div>
			<div>
				<dt class="font-medium text-gray-500 dark:text-gray-400">px4-ulog</dt>
				<dd class="tabular-nums">{backend.px4_ulog}</dd>
			</div>
		{:else if backendFailed}
			<div><dd class="italic">server: unavailable</dd></div>
		{:else}
			<div><dd class="italic opacity-60">loading…</dd></div>
		{/if}
	</dl>
</details>
