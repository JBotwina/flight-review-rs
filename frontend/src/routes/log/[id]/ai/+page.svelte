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
	let selectedModelDetails = $derived(models.find((model) => model.id === selectedModel));

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
	<title>Flight Intelligence - Flight Review</title>
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
	<link href="https://fonts.googleapis.com/css2?family=Barlow+Condensed:wght@500;600;700&family=IBM+Plex+Mono:wght@400;500;600&display=swap" rel="stylesheet" />
</svelte:head>

{#if loading}
	<div class="intel-loading">
		<div class="loading-radar"><LoadingSpinner /></div>
		<p>Synchronizing flight evidence</p>
	</div>
{:else}
	<div class="intelligence-page">
		<section class="control-deck">
			<div class="deck-grid" aria-hidden="true"></div>
			<div class="deck-intro">
				<div class="kicker"><span></span> PX4 / FLIGHT INTELLIGENCE</div>
				<p class="report-number">SECOND OPINION · REPORT CHANNEL 02</p>
				<h2>Interrogate the<br /><em>flight evidence.</em></h2>
				<p class="deck-copy">A model reviews the bounded diagnostic record—not raw position samples—and produces a saved engineering brief. Rust analysis remains the source evidence.</p>
				<div class="evidence-key">
					<span><i>01</i> Modes</span>
					<span><i>02</i> Diagnostics</span>
					<span><i>03</i> Messages</span>
					<span><i>04</i> Field stats</span>
				</div>
			</div>

			<div class="request-console">
				<div class="console-head">
					<div><span class="live-dot"></span> MODEL LINK</div>
					{#if balance}<OpenRouterBalance {balance} variant="console" />{/if}
				</div>

				{#if enabled && models.length > 0}
					<div class="console-body">
						<ModelPicker {models} selected={selectedModel} onSelect={(model) => (selectedModel = model)} disabled={generating} label="Select analysis model" variant="console" />
						{#if selectedModelDetails}
							<div class="model-readout">
								<span>MODEL ID</span>
								<strong>{selectedModelDetails.id}</strong>
								{#if selectedModelDetails.context_length}<b>{Math.round(selectedModelDetails.context_length / 1000)}K CONTEXT</b>{/if}
							</div>
						{/if}
						<button class="analyze-button" type="button" onclick={generate} disabled={generating || !selectedModel}>
							<span>{generating ? 'Analysis in progress' : analysis ? 'Regenerate intelligence brief' : 'Generate intelligence brief'}</span>
							<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12h13m-5-5 5 5-5 5" /></svg>
						</button>
						<p class="cost-note"><span>PAID REQUEST</span> One model call. Regeneration replaces the saved brief.</p>
					</div>
				{:else}
					<div class="console-offline">
						<span>LINK OFFLINE</span>
						<p>OpenRouter is not configured for this environment.</p>
					</div>
				{/if}
			</div>
		</section>

		{#if error}
			<div class="intel-error" role="alert"><span>REQUEST ABORTED</span><p>{error}</p></div>
		{/if}

		{#if generating}
			<div class="analysis-progress">
				<div class="sweep" aria-hidden="true"></div>
				<div class="progress-index">AI / WORKING</div>
				<div class="progress-spinner"><LoadingSpinner /></div>
				<h3>Correlating flight evidence</h3>
				<p>The model is cross-referencing modes, diagnostics, logged messages, parameters, and field statistics.</p>
				<div class="progress-track"><span></span></div>
			</div>
		{:else if analysis}
			<AiAnalysisCard {analysis} />
		{:else}
			<div class="empty-dossier">
				<div class="empty-orbit" aria-hidden="true">
					<svg viewBox="0 0 80 80" fill="none"><circle cx="40" cy="40" r="27"/><circle cx="40" cy="40" r="5"/><path d="M40 4v15M40 61v15M4 40h15M61 40h15"/></svg>
				</div>
				<p class="empty-index">DOSSIER / EMPTY</p>
				<h3>No intelligence brief on file</h3>
				<p>Choose a model in the request console to create an evidence-backed second opinion for this flight.</p>
			</div>
		{/if}

		<footer class="page-caveat"><span>ENGINEERING AID</span> AI output is not an airworthiness determination. Verify findings against plots, messages, and deterministic diagnostics.</footer>
	</div>
{/if}

<style>
	:global(body) { background: #eef1ee; }
	.intelligence-page { --navy: #071823; --ink: #10262d; --paper: #f7f7f2; --acid: #c8ef4b; --cyan: #88d9df; width: min(100%, 88rem); margin: 0 auto; font-family: 'IBM Plex Mono', monospace; color: var(--ink); }
	.control-deck { position: relative; overflow: hidden; display: grid; grid-template-columns: minmax(0, 1.25fr) minmax(24rem, .75fr); min-height: 25rem; border: 1px solid #173641; background: var(--navy); color: #e9f3ef; box-shadow: 0 22px 55px rgba(9,28,37,.16); }
	.deck-grid { position: absolute; inset: 0; opacity: .26; background-image: linear-gradient(rgba(136,217,223,.13) 1px, transparent 1px), linear-gradient(90deg, rgba(136,217,223,.13) 1px, transparent 1px); background-size: 38px 38px; mask-image: linear-gradient(90deg, black, transparent 72%); }
	.deck-grid::after { content: ''; position: absolute; width: 34rem; height: 34rem; left: -9rem; bottom: -23rem; border: 1px solid rgba(200,239,75,.24); border-radius: 50%; box-shadow: 0 0 0 5rem rgba(200,239,75,.025), 0 0 0 10rem rgba(200,239,75,.018); }
	.deck-intro { position: relative; z-index: 1; padding: clamp(2rem, 4vw, 4rem); }
	.kicker { display: flex; align-items: center; gap: .6rem; color: var(--acid); font-size: .62rem; font-weight: 600; letter-spacing: .2em; }
	.kicker span { width: .42rem; height: .42rem; border-radius: 50%; background: var(--acid); box-shadow: 0 0 14px rgba(200,239,75,.7); }
	.report-number { margin: 3.4rem 0 .8rem; color: #6c8e94; font-size: .58rem; letter-spacing: .14em; }
	h2 { margin: 0; font-family: 'Barlow Condensed', sans-serif; font-size: clamp(3.4rem, 6vw, 6rem); font-weight: 600; line-height: .82; letter-spacing: -.03em; text-transform: uppercase; }
	h2 em { color: transparent; font-style: normal; -webkit-text-stroke: 1px rgba(233,243,239,.72); }
	.deck-copy { max-width: 43rem; margin: 1.7rem 0 0; color: #95adb0; font-size: .73rem; line-height: 1.75; }
	.evidence-key { display: flex; flex-wrap: wrap; gap: 1rem 1.4rem; margin-top: 2rem; color: #91a7aa; font-size: .59rem; text-transform: uppercase; }
	.evidence-key i { margin-right: .3rem; color: var(--cyan); font-style: normal; }
	.request-console { position: relative; z-index: 2; margin: 1.25rem; align-self: stretch; border: 1px solid rgba(136,217,223,.22); background: rgba(3,13,20,.72); clip-path: polygon(0 0, calc(100% - 14px) 0, 100% 14px, 100% 100%, 0 100%); }
	.console-head { min-height: 3.25rem; display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: .6rem; padding: .7rem 1rem; border-bottom: 1px solid rgba(136,217,223,.16); color: #8fb4b8; font-size: .6rem; letter-spacing: .16em; }
	.live-dot { display: inline-block; width: .42rem; height: .42rem; margin-right: .35rem; border-radius: 50%; background: var(--acid); box-shadow: 0 0 10px rgba(200,239,75,.75); }
	.console-body { padding: clamp(1.2rem, 2.6vw, 2rem); }
	.model-readout { display: grid; grid-template-columns: auto minmax(0,1fr) auto; gap: .75rem; margin: 1rem 0; padding: .75rem 0; border-top: 1px solid rgba(136,217,223,.12); border-bottom: 1px solid rgba(136,217,223,.12); align-items: center; }
	.model-readout span, .model-readout b { color: #547379; font-size: .52rem; letter-spacing: .12em; }
	.model-readout strong { overflow: hidden; color: #9ab2b4; font-size: .57rem; font-weight: 400; text-overflow: ellipsis; white-space: nowrap; }
	.analyze-button { width: 100%; display: flex; align-items: center; justify-content: space-between; border: 1px solid var(--acid); padding: .95rem 1rem; background: var(--acid); color: #0a1a22; font: 600 .64rem 'IBM Plex Mono', monospace; letter-spacing: .1em; text-transform: uppercase; cursor: pointer; transition: .2s ease; }
	.analyze-button svg { width: 1.2rem; fill: none; stroke: currentColor; stroke-width: 1.8; }
	.analyze-button:hover:not(:disabled) { background: transparent; color: var(--acid); transform: translateY(-1px); }
	.analyze-button:disabled { cursor: not-allowed; opacity: .45; }
	.cost-note { margin: .85rem 0 0; color: #607e83; font-size: .55rem; line-height: 1.55; }
	.cost-note span { color: #e2b361; }
	.console-offline { padding: 2rem; }
	.console-offline span { color: #e98775; font-size: .58rem; letter-spacing: .16em; }
	.console-offline p { color: #8fa3a5; font-size: .72rem; }
	.intel-error { display: grid; grid-template-columns: auto 1fr; gap: 1rem; margin-top: 1rem; border-left: 3px solid #d95f4d; padding: 1rem 1.2rem; background: #fff3ef; color: #792f26; font-size: .72rem; }
	.intel-error span { font-size: .58rem; font-weight: 600; letter-spacing: .12em; }
	.intel-error p { margin: 0; }
	.analysis-progress, .empty-dossier { position: relative; overflow: hidden; margin-top: 1rem; border: 1px solid #cbd3cf; background: var(--paper); text-align: center; }
	.analysis-progress { min-height: 25rem; display: grid; place-items: center; align-content: center; padding: 3rem; }
	.sweep { position: absolute; inset: 0; background: linear-gradient(110deg, transparent 38%, rgba(65,129,133,.08) 49%, transparent 60%); animation: sweep 2.8s linear infinite; }
	.progress-index, .empty-index { color: #758889; font-size: .58rem; letter-spacing: .19em; }
	.progress-spinner { width: 3.3rem; height: 3.3rem; display: grid; place-items: center; margin: 1.3rem auto; border: 1px solid #b6c4c0; color: #12636c; transform: rotate(45deg); }
	.progress-spinner :global(*) { transform: rotate(-45deg); }
	.analysis-progress h3, .empty-dossier h3 { margin: 0; font-family: 'Barlow Condensed', sans-serif; font-size: 2rem; font-weight: 600; text-transform: uppercase; }
	.analysis-progress p, .empty-dossier > p:last-child { max-width: 36rem; margin: .7rem auto 0; color: #687b7c; font-size: .7rem; line-height: 1.7; }
	.progress-track { width: min(24rem, 70vw); height: 2px; margin-top: 2rem; background: #d5dcda; overflow: hidden; }
	.progress-track span { display: block; width: 36%; height: 100%; background: #176973; animation: progress 1.7s ease-in-out infinite alternate; }
	.empty-dossier { min-height: 24rem; display: grid; place-items: center; align-content: center; padding: 3rem; background-image: linear-gradient(rgba(30,68,72,.04) 1px, transparent 1px), linear-gradient(90deg, rgba(30,68,72,.04) 1px, transparent 1px); background-size: 30px 30px; }
	.empty-orbit { width: 5rem; height: 5rem; margin-bottom: 1.2rem; color: #7c9998; }
	.empty-orbit svg { width: 100%; stroke: currentColor; stroke-width: 1; }
	.page-caveat { display: flex; gap: .8rem; padding: 1rem .2rem .2rem; color: #778587; font-size: .56rem; line-height: 1.55; }
	.page-caveat span { flex: none; color: #385b5f; font-weight: 600; letter-spacing: .1em; }
	.intel-loading { min-height: 28rem; display: grid; place-items: center; align-content: center; background: #071823; color: #91aeb0; font: 500 .62rem 'IBM Plex Mono', monospace; letter-spacing: .15em; text-transform: uppercase; }
	.loading-radar { width: 3rem; height: 3rem; display: grid; place-items: center; margin-bottom: 1rem; color: #c8ef4b; }
	@keyframes sweep { from { transform: translateX(-70%); } to { transform: translateX(70%); } }
	@keyframes progress { from { transform: translateX(-20%); } to { transform: translateX(195%); } }
	@media (max-width: 900px) { .control-deck { grid-template-columns: 1fr; } .request-console { margin-top: 0; } .report-number { margin-top: 2.4rem; } }
	@media (max-width: 520px) { .deck-intro { padding: 1.5rem; } h2 { font-size: 3.3rem; } .request-console { margin: .7rem; } .model-readout { grid-template-columns: 1fr; gap: .35rem; } .model-readout b { display: none; } .page-caveat { display: block; } .page-caveat span { display: block; margin-bottom: .35rem; } }
	@media (prefers-reduced-motion: reduce) { .sweep, .progress-track span { animation: none; } }
</style>
