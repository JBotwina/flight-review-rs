<script lang="ts">
	import type { AiModel } from '$lib/types';

	let {
		models,
		selected,
		onSelect,
		disabled = false,
		label = 'OpenRouter model',
	}: {
		models: AiModel[];
		selected: string;
		onSelect: (model: string) => void;
		disabled?: boolean;
		label?: string;
	} = $props();

	let open = $state(false);
	let query = $state('');
	let blurTimer: ReturnType<typeof setTimeout> | undefined;

	const selectedModel = $derived(models.find((model) => model.id === selected));
	const filtered = $derived.by(() => {
		const needle = query.trim().toLowerCase();
		const matches = needle
			? models.filter((model) => `${model.name} ${model.id}`.toLowerCase().includes(needle))
			: models;
		return matches.slice(0, 80);
	});

	function choose(model: AiModel) {
		onSelect(model.id);
		query = '';
		open = false;
	}

	function handleBlur() {
		blurTimer = setTimeout(() => (open = false), 150);
	}

	function handleFocus() {
		if (blurTimer) clearTimeout(blurTimer);
		open = true;
	}

	function pricePerMillion(value: string | null | undefined): string | null {
		if (!value) return null;
		const price = Number(value) * 1_000_000;
		if (!Number.isFinite(price)) return null;
		return `$${price < 0.01 ? price.toFixed(3) : price.toFixed(2)}/M`;
	}
</script>

<div class="relative">
	<label for="ai-model-search" class="mb-1.5 block text-xs font-semibold uppercase tracking-[0.12em] text-slate-500">
		{label}
	</label>
	<div class="relative">
		<input
			id="ai-model-search"
			type="text"
			value={open ? query : (selectedModel?.name ?? selected)}
			oninput={(event) => { query = event.currentTarget.value; open = true; }}
			onfocus={handleFocus}
			onblur={handleBlur}
			onkeydown={(event) => { if (event.key === 'Escape') open = false; }}
			placeholder="Search available models…"
			disabled={disabled}
			role="combobox"
			aria-expanded={open}
			aria-controls="ai-model-options"
			autocomplete="off"
			class="block w-full rounded-md border-0 bg-white py-2.5 pl-3 pr-10 text-sm font-medium text-slate-900 shadow-sm ring-1 ring-inset ring-slate-300 placeholder:text-slate-400 focus:ring-2 focus:ring-inset focus:ring-sky-600 disabled:opacity-50"
		/>
		<svg class="pointer-events-none absolute right-3 top-3 size-4 text-slate-400" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
			<path fill-rule="evenodd" d="M5.22 8.22a.75.75 0 011.06 0L10 11.94l3.72-3.72a.75.75 0 111.06 1.06l-4.25 4.25a.75.75 0 01-1.06 0L5.22 9.28a.75.75 0 010-1.06z" clip-rule="evenodd" />
		</svg>
	</div>

	{#if selectedModel && !open}
		<p class="mt-1.5 truncate text-xs text-slate-500">
			{selectedModel.id}
			{#if pricePerMillion(selectedModel.pricing?.prompt)}
				<span class="mx-1 text-slate-300">·</span>
				{pricePerMillion(selectedModel.pricing?.prompt)} input
			{/if}
		</p>
	{/if}

	{#if open && !disabled}
		<div id="ai-model-options" role="listbox" class="absolute z-30 mt-1 max-h-80 w-full overflow-auto rounded-md bg-white py-1 shadow-xl ring-1 ring-slate-900/10">
			{#if filtered.length === 0}
				<div class="px-3 py-5 text-center text-sm text-slate-500">No available model matches “{query}”.</div>
			{:else}
				{#each filtered as model}
					<button
						type="button"
						role="option"
						aria-selected={model.id === selected}
						onmousedown={(event) => { event.preventDefault(); choose(model); }}
						class="block w-full px-3 py-2.5 text-left hover:bg-sky-50 {model.id === selected ? 'bg-sky-50' : ''}"
					>
						<span class="flex items-center justify-between gap-3">
							<span class="truncate text-sm font-semibold text-slate-900">{model.name}</span>
							{#if model.context_length}
								<span class="shrink-0 text-[11px] tabular-nums text-slate-400">{Math.round(model.context_length / 1000)}k ctx</span>
							{/if}
						</span>
						<span class="mt-0.5 block truncate font-mono text-[11px] text-slate-500">{model.id}</span>
					</button>
				{/each}
				{#if filtered.length === 80}
					<div class="border-t border-slate-100 px-3 py-2 text-center text-xs text-slate-400">Type to narrow the list</div>
				{/if}
			{/if}
		</div>
	{/if}
</div>
