<script lang="ts">
	import { getContext, onMount } from 'svelte';
	import { ApiError, generateAiAnalysis, getAiAnalysis, getAiBalance, getAiModels } from '$lib/api';
	import type { AiAnalysis, AiBalanceResponse, AiModel } from '$lib/types';
	import AiAnalysisCard from '$lib/components/ai/AiAnalysisCard.svelte';
	import ModelPicker from '$lib/components/ai/ModelPicker.svelte';
	import OpenRouterBalance from '$lib/components/ai/OpenRouterBalance.svelte';
	import LoadingSpinner from '$lib/components/shared/LoadingSpinner.svelte';

	const ctx = getContext<{ logId: string }>('log-viewer');

	let analysis = $state<AiAnalysis | null>(null);
	let models = $state<AiModel[]>([]);
	let balance = $state<AiBalanceResponse | null>(null);
	let selectedModel = $state('');
	let enabled = $state(false);
	let loading = $state(true);
	let generating = $state(false);
	let error = $state('');

	function messageFrom(error: unknown): string {
		if (error instanceof ApiError) {
			if (error.status === 404) return '';
			return error.message;
		}
		return error instanceof Error ? error.message : 'AI analysis failed.';
	}

	onMount(async () => {
		const [modelsResult, analysisResult, balanceResult] = await Promise.allSettled([
			getAiModels(),
			getAiAnalysis(ctx.logId),
			getAiBalance(),
		]);

		if (modelsResult.status === 'fulfilled') {
			enabled = modelsResult.value.enabled;
			models = modelsResult.value.models;
			selectedModel = modelsResult.value.default_model ?? models[0]?.id ?? '';
		} else {
			error = messageFrom(modelsResult.reason);
		}

		if (analysisResult.status === 'fulfilled') {
			analysis = analysisResult.value;
			selectedModel = analysis.requested_model || selectedModel;
		} else {
			const analysisError = messageFrom(analysisResult.reason);
			if (analysisError) error = analysisError;
		}
		if (balanceResult.status === 'fulfilled') balance = balanceResult.value;
		loading = false;
	});

	async function generate() {
		if (!selectedModel) return;
		generating = true;
		error = '';
		try {
			analysis = await generateAiAnalysis(ctx.logId, selectedModel);
			try {
				balance = await getAiBalance();
			} catch {
				// Balance is supplemental; a refresh will retry it.
			}
		} catch (caught) {
			error = messageFrom(caught);
		} finally {
			generating = false;
		}
	}
</script>

<svelte:head>
	<title>AI Analysis - Flight Review</title>
</svelte:head>

{#if loading}
	<div class="flex min-h-64 items-center justify-center"><LoadingSpinner /></div>
{:else}
	<div class="mx-auto w-full max-w-6xl space-y-4 py-1">
		<div class="flex flex-col gap-4 rounded-xl border border-slate-200 bg-white p-4 shadow-sm sm:flex-row sm:items-end sm:justify-between">
			<div class="max-w-2xl">
				<div class="flex flex-wrap items-center gap-2">
					<p class="text-[11px] font-bold uppercase tracking-[0.16em] text-sky-700">OpenRouter model control</p>
					{#if balance}<OpenRouterBalance {balance} />{/if}
				</div>
				<h2 class="mt-1 text-lg font-bold text-slate-900">Run a second-opinion flight review</h2>
				<p class="mt-1 text-sm leading-5 text-slate-500">Choose any text model available to your API key. Regenerating replaces the saved brief; the deterministic Rust diagnostics remain unchanged.</p>
			</div>
			{#if enabled && models.length > 0}
				<div class="w-full sm:w-[28rem]">
					<div class="flex items-end gap-2">
						<div class="min-w-0 flex-1">
							<ModelPicker {models} selected={selectedModel} onSelect={(model) => (selectedModel = model)} disabled={generating} label="Analysis model" />
						</div>
						<button
							type="button"
							onclick={generate}
							disabled={generating || !selectedModel}
							class="mb-[1.4rem] shrink-0 rounded-md bg-slate-900 px-4 py-2.5 text-sm font-bold text-white shadow-sm hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-50"
						>
							{generating ? 'Analyzing…' : analysis ? 'Regenerate' : 'Analyze'}
						</button>
					</div>
				</div>
			{:else}
				<div class="rounded-md bg-slate-100 px-3 py-2 text-xs text-slate-600">Set <code class="font-mono">OPENROUTER_API_KEY</code> to generate analysis.</div>
			{/if}
		</div>

		{#if error}
			<div class="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700" role="alert">{error}</div>
		{/if}

		{#if generating}
			<div class="relative overflow-hidden rounded-xl border border-sky-200 bg-sky-50 px-6 py-12 text-center">
				<div class="absolute inset-x-0 top-0 h-0.5 animate-pulse bg-sky-500"></div>
				<div class="mx-auto mb-3 flex size-10 items-center justify-center rounded-full bg-slate-900 text-sky-300"><LoadingSpinner /></div>
				<p class="text-sm font-bold text-slate-900">Reviewing flight evidence</p>
				<p class="mt-1 text-xs text-slate-500">The model is correlating Rust diagnostics, messages, modes, and field statistics.</p>
			</div>
		{:else if analysis}
			<AiAnalysisCard {analysis} />
		{:else}
			<div class="rounded-xl border border-dashed border-slate-300 bg-white px-6 py-16 text-center">
				<div class="mx-auto flex size-12 items-center justify-center rounded-lg bg-slate-900 text-sky-300">
					<svg class="size-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" d="M8 3v3m8-3v3M3 8h3m12 0h3M5 12a7 7 0 1014 0 7 7 0 00-14 0z" /><path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6m-3-3v6" /></svg>
				</div>
				<h3 class="mt-4 text-base font-bold text-slate-900">No AI debrief saved yet</h3>
				<p class="mx-auto mt-2 max-w-md text-sm leading-6 text-slate-500">Select a model above to summarize the flight, surface anomalies, and suggest focused follow-up checks.</p>
			</div>
		{/if}

		<p class="px-1 text-[11px] leading-5 text-slate-400">AI output is an engineering aid, not an airworthiness determination. Verify important findings against plots, messages, and the deterministic diagnostics.</p>
	</div>
{/if}
