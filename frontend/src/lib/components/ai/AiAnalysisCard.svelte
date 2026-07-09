<script lang="ts">
	import type { AiAnalysis, AiFindingSeverity, AiRiskLevel } from '$lib/types';

	let { analysis, compact = false }: { analysis: AiAnalysis; compact?: boolean } = $props();

	const visibleFindings = $derived(compact ? analysis.findings.slice(0, 3) : analysis.findings);

	function riskStyle(risk: AiRiskLevel): string {
		switch (risk) {
			case 'low': return 'bg-emerald-100 text-emerald-800 ring-emerald-600/20';
			case 'moderate': return 'bg-amber-100 text-amber-800 ring-amber-600/20';
			case 'high': return 'bg-orange-100 text-orange-800 ring-orange-600/20';
			case 'critical': return 'bg-red-100 text-red-800 ring-red-600/20';
			default: return 'bg-slate-100 text-slate-700 ring-slate-500/20';
		}
	}

	function severityStyle(severity: AiFindingSeverity): { rail: string; badge: string } {
		switch (severity) {
			case 'critical': return { rail: 'border-l-red-500', badge: 'bg-red-50 text-red-700 ring-red-600/20' };
			case 'warning': return { rail: 'border-l-amber-500', badge: 'bg-amber-50 text-amber-700 ring-amber-600/20' };
			default: return { rail: 'border-l-sky-500', badge: 'bg-sky-50 text-sky-700 ring-sky-600/20' };
		}
	}

	function timeLabel(start: number, end: number | null): string {
		return end == null ? `T+${start.toFixed(1)}s` : `T+${start.toFixed(1)}–${end.toFixed(1)}s`;
	}
</script>

<section class="overflow-hidden rounded-xl border border-slate-200 bg-white shadow-sm">
	<div class="relative overflow-hidden bg-slate-950 px-5 py-5 text-white sm:px-6">
		<div class="pointer-events-none absolute inset-0 opacity-20" style="background-image: linear-gradient(rgba(56,189,248,.25) 1px, transparent 1px), linear-gradient(90deg, rgba(56,189,248,.25) 1px, transparent 1px); background-size: 24px 24px;"></div>
		<div class="relative flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
			<div class="max-w-3xl">
				<div class="mb-2 flex items-center gap-2 text-[11px] font-semibold uppercase tracking-[0.18em] text-sky-300">
					<span class="inline-block size-1.5 rounded-full bg-sky-300 shadow-[0_0_10px_rgba(125,211,252,.8)]"></span>
					AI flight debrief
				</div>
				<p class="text-sm leading-6 text-slate-200 sm:text-base">{analysis.summary}</p>
			</div>
			<div class="flex shrink-0 items-center gap-2 sm:flex-col sm:items-end">
				<span class="inline-flex items-center rounded-md px-2.5 py-1 text-xs font-bold uppercase tracking-wide ring-1 ring-inset {riskStyle(analysis.risk_level)}">
					{analysis.risk_level} risk
				</span>
				{#if analysis.confidence != null}
					<span class="text-[11px] tabular-nums text-slate-400">{Math.round(analysis.confidence * 100)}% confidence</span>
				{/if}
			</div>
		</div>
	</div>

	<div class="p-5 sm:p-6">
		{#if visibleFindings.length > 0}
			<div class="space-y-3">
				{#each visibleFindings as finding}
					{@const style = severityStyle(finding.severity)}
					<article class="rounded-lg border border-slate-200 border-l-4 bg-slate-50/60 p-4 {style.rail}">
						<div class="flex flex-wrap items-center gap-2">
							<h3 class="text-sm font-semibold text-slate-900">{finding.title}</h3>
							<span class="rounded px-1.5 py-0.5 text-[10px] font-bold uppercase tracking-wide ring-1 ring-inset {style.badge}">{finding.severity}</span>
							<span class="text-[10px] font-semibold uppercase tracking-wide text-slate-400">{finding.category}</span>
							{#if finding.time_range_s}
								<span class="ml-auto font-mono text-[11px] text-slate-500">{timeLabel(finding.time_range_s.start, finding.time_range_s.end)}</span>
							{/if}
						</div>
						<p class="mt-2 text-sm leading-6 text-slate-600">{finding.explanation}</p>
						{#if finding.evidence.length > 0}
							<ul class="mt-3 grid gap-1.5 sm:grid-cols-2">
								{#each finding.evidence as evidence}
									<li class="flex gap-2 font-mono text-[11px] leading-5 text-slate-600">
										<span class="mt-2 size-1 shrink-0 rounded-full bg-sky-500"></span>{evidence}
									</li>
								{/each}
							</ul>
						{/if}
					</article>
				{/each}
			</div>
		{:else}
			<div class="rounded-lg border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
				No anomaly findings were produced from the supplied flight evidence.
			</div>
		{/if}

		{#if !compact}
			<div class="mt-6 grid gap-6 lg:grid-cols-2">
				{#if analysis.positive_observations.length > 0}
					<section>
						<h3 class="text-xs font-bold uppercase tracking-[0.14em] text-slate-500">Positive observations</h3>
						<ul class="mt-3 space-y-2">
							{#each analysis.positive_observations as observation}
								<li class="flex gap-2.5 text-sm leading-5 text-slate-700">
									<svg class="mt-0.5 size-4 shrink-0 text-emerald-500" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true"><path fill-rule="evenodd" d="M16.704 4.153a.75.75 0 01.143 1.052l-8 10.5a.75.75 0 01-1.127.075l-4.5-4.5a.75.75 0 011.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 011.05-.143z" clip-rule="evenodd" /></svg>
									{observation}
								</li>
							{/each}
						</ul>
					</section>
				{/if}

				{#if analysis.recommendations.length > 0}
					<section>
						<h3 class="text-xs font-bold uppercase tracking-[0.14em] text-slate-500">Recommended follow-up</h3>
						<ol class="mt-3 space-y-3">
							{#each analysis.recommendations as recommendation, index}
								<li class="flex gap-3">
									<span class="flex size-6 shrink-0 items-center justify-center rounded bg-slate-900 font-mono text-xs font-bold text-sky-300">{index + 1}</span>
									<div>
										<p class="text-sm font-semibold text-slate-900">{recommendation.action}</p>
										<p class="mt-0.5 text-xs leading-5 text-slate-500">{recommendation.rationale}</p>
									</div>
								</li>
							{/each}
						</ol>
					</section>
				{/if}
			</div>

			{#if analysis.limitations.length > 0}
				<details class="mt-6 border-t border-slate-200 pt-4">
					<summary class="cursor-pointer text-xs font-semibold text-slate-500 hover:text-slate-700">Analysis limitations</summary>
					<ul class="mt-2 list-disc space-y-1 pl-5 text-xs leading-5 text-slate-500">
						{#each analysis.limitations as limitation}<li>{limitation}</li>{/each}
					</ul>
				</details>
			{/if}
		{/if}

		<footer class="mt-5 flex flex-wrap items-center gap-x-3 gap-y-1 border-t border-slate-100 pt-3 text-[11px] text-slate-400">
			<span>Model <span class="font-mono text-slate-500">{analysis.model}</span></span>
			<span>·</span>
			<time datetime={analysis.generated_at}>{new Date(analysis.generated_at).toLocaleString()}</time>
			{#if analysis.usage?.total_tokens}
				<span>·</span><span>{analysis.usage.total_tokens.toLocaleString()} tokens</span>
			{/if}
		</footer>
	</div>
</section>
