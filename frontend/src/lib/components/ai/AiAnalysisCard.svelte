<script lang="ts">
	import type { AiAnalysis, AiFindingSeverity, AiRiskLevel } from '$lib/types';
	import { downloadAiAnalysisPdf } from '$lib/utils/aiPdf';

	let { analysis, compact = false, reportId }: { analysis: AiAnalysis; compact?: boolean; reportId?: string } = $props();
	let pdfGenerating = $state(false);
	let pdfError = $state('');

	const visibleFindings = $derived(compact ? analysis.findings.slice(0, 3) : analysis.findings);
	const confidencePercent = $derived(analysis.confidence == null ? null : Math.round(analysis.confidence * 100));

	function riskLabel(risk: AiRiskLevel): string {
		return risk === 'unknown' ? 'Unrated' : `${risk} risk`;
	}

	function severityCode(severity: AiFindingSeverity): string {
		switch (severity) {
			case 'critical': return 'CRT';
			case 'warning': return 'WRN';
			case 'info': return 'INF';
			default: return 'UNK';
		}
	}

	function timeLabel(start: number, end: number | null): string {
		return end == null ? `T+${start.toFixed(1)}s` : `T+${start.toFixed(1)}–${end.toFixed(1)}s`;
	}

	async function downloadPdf() {
		pdfGenerating = true;
		pdfError = '';
		try {
			await downloadAiAnalysisPdf(analysis, reportId);
		} catch (error) {
			pdfError = error instanceof Error ? error.message : String(error || 'PDF generation failed.');
		} finally {
			pdfGenerating = false;
		}
	}
</script>

{#if compact}
	<section class="compact-brief">
		<header>
			<div>
				<p>AI FLIGHT BRIEF</p>
				<h3>{riskLabel(analysis.risk_level)}</h3>
			</div>
			{#if confidencePercent != null}<span>{confidencePercent}% confidence</span>{/if}
		</header>
		<p class="compact-summary">{analysis.summary}</p>
		{#if visibleFindings.length > 0}
			<div class="compact-findings">
				{#each visibleFindings as finding}
					<div data-severity={finding.severity}><b>{severityCode(finding.severity)}</b><span>{finding.title}</span></div>
				{/each}
			</div>
		{/if}
	</section>
{:else}
	<article class="dossier">
		<div class="document-rail">
			<div class="rail-metadata">
				<span>PX4 / FLIGHT INTELLIGENCE</span>
				<span>GENERATED {new Date(analysis.generated_at).toLocaleDateString()}</span>
				<span>SCHEMA {analysis.schema_version}</span>
			</div>
			<button class="pdf-download" type="button" onclick={downloadPdf} disabled={pdfGenerating} aria-describedby={pdfError ? 'pdf-export-error' : undefined}>
				<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3v12m0 0 4-4m-4 4-4-4M5 19h14" /></svg>
				<span>{pdfGenerating ? 'BUILDING PDF' : 'DOWNLOAD PDF'}</span>
				<small>WASM / LOCAL</small>
			</button>
		</div>
		{#if pdfError}<p class="pdf-error" id="pdf-export-error" role="alert">PDF EXPORT FAILED / {pdfError}</p>{/if}

		<header class="brief-header">
			<div class="summary-block">
				<p class="section-code">00 — EXECUTIVE READOUT</p>
				<h2>Second-opinion<br />flight brief</h2>
				<p class="summary">{analysis.summary}</p>
			</div>

			<div class="risk-stamp risk-{analysis.risk_level}">
				<div class="crosshair" aria-hidden="true"><span></span><span></span></div>
				<p>ASSESSMENT</p>
				<strong>{analysis.risk_level}</strong>
				<span>RISK LEVEL</span>
				{#if confidencePercent != null}
					<div class="confidence">
						<div><i style={`width: ${confidencePercent}%`}></i></div>
						<b>{confidencePercent}% CONFIDENCE</b>
					</div>
				{/if}
			</div>
		</header>

		<section class="findings-section">
			<div class="section-heading">
				<div>
					<p class="section-code">01 — OBSERVATIONS</p>
					<h3>Evidence-backed findings</h3>
				</div>
				<span class="count">{String(analysis.findings.length).padStart(2, '0')} FINDINGS</span>
			</div>

			{#if visibleFindings.length > 0}
				<div class="finding-list">
					{#each visibleFindings as finding, index}
						<section class="finding" data-severity={finding.severity}>
							<div class="finding-index">{String(index + 1).padStart(2, '0')}</div>
							<div class="finding-main">
								<div class="finding-meta">
									<span class="severity">{severityCode(finding.severity)} / {finding.severity}</span>
									<span>{finding.category}</span>
									{#if finding.time_range_s}<time>{timeLabel(finding.time_range_s.start, finding.time_range_s.end)}</time>{/if}
								</div>
								<h4>{finding.title}</h4>
								<p class="explanation">{finding.explanation}</p>
								{#if finding.evidence.length > 0}
									<div class="evidence-block">
										<p>SUPPORTING EVIDENCE</p>
										<ul>
											{#each finding.evidence as evidence}<li><span></span>{evidence}</li>{/each}
										</ul>
									</div>
								{/if}
							</div>
						</section>
					{/each}
				</div>
			{:else}
				<div class="clear-state"><span>✓</span><div><strong>No anomaly findings</strong><p>The supplied evidence did not produce a reportable anomaly.</p></div></div>
			{/if}
		</section>

		<div class="follow-up-grid">
			<section class="positive-panel">
				<p class="section-code">02 — NOMINAL SIGNALS</p>
				<h3>Positive observations</h3>
				{#if analysis.positive_observations.length > 0}
					<ul>{#each analysis.positive_observations as observation, index}<li><span>{String(index + 1).padStart(2, '0')}</span>{observation}</li>{/each}</ul>
				{:else}<p class="panel-empty">No positive observations were returned.</p>{/if}
			</section>

			<section class="recommendation-panel">
				<p class="section-code">03 — NEXT ACTIONS</p>
				<h3>Recommended follow-up</h3>
				{#if analysis.recommendations.length > 0}
					<ol>
						{#each analysis.recommendations as recommendation, index}
							<li>
								<div class="action-index">{index + 1}</div>
								<div><span>{recommendation.priority} priority</span><strong>{recommendation.action}</strong><p>{recommendation.rationale}</p></div>
							</li>
						{/each}
					</ol>
				{:else}<p class="panel-empty">No follow-up actions were returned.</p>{/if}
			</section>
		</div>

		{#if analysis.limitations.length > 0}
			<details class="limitations">
				<summary><span>04 — ANALYSIS ENVELOPE</span> Known limitations <b>+</b></summary>
				<ul>{#each analysis.limitations as limitation}<li>{limitation}</li>{/each}</ul>
			</details>
		{/if}

		<footer class="document-footer">
			<div><span>MODEL</span><strong>{analysis.model}</strong></div>
			<div><span>UTC GENERATED</span><strong>{new Date(analysis.generated_at).toISOString().replace('T', ' ').replace('.000Z', 'Z')}</strong></div>
			{#if analysis.usage?.total_tokens}<div><span>TOKEN LOAD</span><strong>{analysis.usage.total_tokens.toLocaleString()}</strong></div>{/if}
		</footer>
	</article>
{/if}

<style>
	.dossier { --ink: #10262d; --body: #1a3339; --muted: #5a6e71; --line: #cbd2ce; --paper: #f8f8f3; --prose: 'IBM Plex Sans', ui-sans-serif, system-ui, sans-serif; margin-top: 1rem; border: 1px solid #bdc8c4; background: var(--paper); color: var(--ink); box-shadow: 0 18px 50px rgba(22,48,54,.12); font-family: 'IBM Plex Mono', monospace; }
	.document-rail { min-height: 2.7rem; display: flex; align-items: center; justify-content: space-between; gap: 1rem; padding: .55rem .65rem .55rem 1.2rem; border-bottom: 1px solid var(--line); color: #5f7376; font-size: .55rem; letter-spacing: .13em; }
	.rail-metadata { flex: 1; display: flex; align-items: center; justify-content: space-between; gap: 1rem; }
	.pdf-download { flex: none; display: grid; grid-template-columns: 1rem auto; align-items: center; gap: .05rem .55rem; border: 1px solid #244e54; padding: .45rem .65rem; background: #102830; color: #e9f2ee; font: 600 .52rem 'IBM Plex Mono', monospace; letter-spacing: .08em; cursor: pointer; transition: .18s ease; }
	.pdf-download svg { grid-row: 1 / 3; width: .95rem; fill: none; stroke: #c8ef4b; stroke-width: 1.7; stroke-linecap: square; stroke-linejoin: miter; }
	.pdf-download small { color: #82a0a1; font: 500 .4rem 'IBM Plex Mono', monospace; letter-spacing: .1em; }
	.pdf-download:hover:not(:disabled) { border-color: #102830; background: #c8ef4b; color: #102830; transform: translateY(-1px); }
	.pdf-download:hover:not(:disabled) svg { stroke: #102830; }
	.pdf-download:hover:not(:disabled) small { color: #36565a; }
	.pdf-download:disabled { cursor: wait; opacity: .58; }
	.pdf-error { margin: 0; border-bottom: 1px solid #e1b7ad; padding: .65rem 1.2rem; background: #fff1ed; color: #9b3d30; font-size: .57rem; letter-spacing: .06em; }
	.brief-header { display: grid; grid-template-columns: minmax(0,1fr) 18rem; border-bottom: 1px solid var(--line); }
	.summary-block { padding: clamp(2rem, 4vw, 4rem); }
	.section-code { margin: 0 0 .8rem; color: #2f5f63; font-size: .58rem; font-weight: 600; letter-spacing: .16em; }
	.summary-block h2 { margin: 0; font-family: 'Barlow Condensed', sans-serif; font-size: clamp(3.2rem, 6vw, 5.4rem); font-weight: 600; line-height: .84; letter-spacing: -.035em; text-transform: uppercase; }
	.summary { max-width: 46rem; margin: 2rem 0 0; color: var(--body); font: 400 clamp(.95rem, 1.35vw, 1.08rem)/1.7 var(--prose); letter-spacing: -.01em; }
	.risk-stamp { position: relative; min-height: 19rem; display: flex; flex-direction: column; align-items: center; justify-content: center; border-left: 1px solid var(--line); background: #e9ece7; text-align: center; overflow: hidden; }
	.risk-stamp::before { content: ''; position: absolute; width: 12rem; height: 12rem; border: 1px solid currentColor; border-radius: 50%; opacity: .13; }
	.crosshair span { position: absolute; left: 50%; top: 50%; width: 15rem; height: 1px; background: currentColor; opacity: .09; transform: translate(-50%,-50%); }
	.crosshair span:last-child { transform: translate(-50%,-50%) rotate(90deg); }
	.risk-stamp > p, .risk-stamp > span { z-index: 1; margin: 0; font-size: .58rem; letter-spacing: .18em; }
	.risk-stamp > strong { z-index: 1; font-family: 'Barlow Condensed', sans-serif; font-size: 3.6rem; line-height: 1; text-transform: uppercase; }
	.risk-low { color: #167258; } .risk-moderate { color: #9b6717; } .risk-high { color: #a94723; } .risk-critical { color: #a12f2b; } .risk-unknown { color: #52686a; }
	.confidence { z-index: 1; width: 10rem; margin-top: 1.4rem; }
	.confidence > div { height: 2px; background: currentColor; opacity: .25; }
	.confidence i { display: block; height: 100%; background: currentColor; opacity: 1; }
	.confidence b { display: block; margin-top: .45rem; font-size: .56rem; letter-spacing: .13em; }
	.findings-section { padding: clamp(1.5rem, 3.5vw, 3.5rem); }
	.section-heading { display: flex; align-items: end; justify-content: space-between; gap: 1rem; margin-bottom: 1.5rem; }
	.section-heading h3, .follow-up-grid h3 { margin: 0; font-family: 'Barlow Condensed', sans-serif; font-size: 2rem; font-weight: 600; text-transform: uppercase; }
	.count { color: #5a6e71; font-size: .56rem; letter-spacing: .14em; }
	.finding-list { border-top: 1px solid var(--ink); }
	.finding { display: grid; grid-template-columns: 4rem minmax(0,1fr); border-bottom: 1px solid var(--line); }
	.finding-index { padding: 1.35rem .8rem; border-right: 1px solid var(--line); color: #849193; font-family: 'Barlow Condensed', sans-serif; font-size: 1.6rem; }
	.finding-main { padding: 1.35rem 1.5rem 1.6rem; border-left: 3px solid #4f8990; }
	.finding[data-severity='warning'] .finding-main { border-left-color: #d59b35; }
	.finding[data-severity='critical'] .finding-main { border-left-color: #d25449; }
	.finding-meta { display: flex; flex-wrap: wrap; align-items: center; gap: .6rem 1rem; color: #5f7376; font-size: .54rem; letter-spacing: .12em; text-transform: uppercase; }
	.finding-meta .severity { color: #326f76; font-weight: 600; }
	.finding[data-severity='warning'] .severity { color: #a56b08; }
	.finding[data-severity='critical'] .severity { color: #b43d34; }
	.finding-meta time { margin-left: auto; letter-spacing: .04em; }
	.finding h4 { margin: .65rem 0 0; font-family: 'Barlow Condensed', sans-serif; font-size: 1.55rem; font-weight: 600; }
	.explanation { margin: .7rem 0 0; color: var(--body); font: 400 .92rem/1.65 var(--prose); letter-spacing: -.01em; }
	.evidence-block { margin-top: 1rem; padding: .85rem 1rem; border: 1px solid #d4d9d5; background: #eff1ec; }
	.evidence-block > p { margin: 0 0 .55rem; color: #5a6e71; font-size: .52rem; letter-spacing: .14em; }
	.evidence-block ul { margin: 0; padding: 0; display: grid; gap: .4rem .9rem; grid-template-columns: repeat(2,minmax(0,1fr)); list-style: none; }
	.evidence-block li { display: flex; gap: .55rem; color: var(--body); font: 400 .8rem/1.55 var(--prose); letter-spacing: -.005em; }
	.evidence-block li span { width: .3rem; height: .3rem; flex: none; margin-top: .4rem; background: #37777d; transform: rotate(45deg); }
	.clear-state { display: flex; gap: 1rem; align-items: center; border: 1px solid #a8cfc1; padding: 1rem; background: #eef6f0; color: #26634f; }
	.clear-state > span { font-size: 1.3rem; } .clear-state strong { font: 600 .9rem var(--prose); } .clear-state p { margin: .25rem 0 0; font: 400 .82rem/1.5 var(--prose); }
	.follow-up-grid { display: grid; grid-template-columns: .9fr 1.1fr; border-top: 1px solid var(--line); }
	.follow-up-grid > section { padding: clamp(1.5rem,3vw,3rem); }
	.positive-panel { background: #102830; color: #e9f2ee; }
	.positive-panel .section-code { color: #8fd2c0; }
	.positive-panel ul { margin: 1.5rem 0 0; padding: 0; list-style: none; }
	.positive-panel li { display: grid; grid-template-columns: 2rem 1fr; gap: .75rem; padding: .85rem 0; border-top: 1px solid rgba(181,216,208,.15); color: #d5e3e1; font: 400 .88rem/1.65 var(--prose); letter-spacing: -.005em; }
	.positive-panel li span { color: #8fd2c0; font: 600 .55rem 'IBM Plex Mono', monospace; }
	.recommendation-panel { border-left: 1px solid var(--line); background: #eceee9; }
	.recommendation-panel ol { margin: 1.5rem 0 0; padding: 0; list-style: none; }
	.recommendation-panel li { display: grid; grid-template-columns: 2.2rem 1fr; gap: .9rem; padding: 1rem 0; border-top: 1px solid #c9d0cc; }
	.action-index { width: 1.8rem; height: 1.8rem; display: grid; place-items: center; background: #102830; color: #c8ef4b; font-size: .58rem; }
	.recommendation-panel li span { display: block; color: #7f6a3d; font-size: .52rem; letter-spacing: .12em; text-transform: uppercase; }
	.recommendation-panel li strong { display: block; margin-top: .25rem; font: 600 1.15rem 'Barlow Condensed', sans-serif; }
	.recommendation-panel li p { margin: .4rem 0 0; color: var(--body); font: 400 .86rem/1.6 var(--prose); letter-spacing: -.005em; }
	.panel-empty { color: var(--muted); font: 400 .84rem/1.5 var(--prose); }
	.limitations { border-top: 1px solid var(--line); }
	.limitations summary { display: flex; align-items: center; gap: 1rem; padding: 1rem 1.3rem; color: #3f5558; font: 500 .72rem var(--prose); cursor: pointer; list-style: none; }
	.limitations summary span { color: #2f5f63; font: 600 .52rem 'IBM Plex Mono', monospace; letter-spacing: .14em; }
	.limitations summary b { margin-left: auto; font-size: 1rem; }
	.limitations ul { margin: 0; padding: 0 3rem 1.3rem; color: var(--body); font: 400 .84rem/1.7 var(--prose); letter-spacing: -.005em; }
	.document-footer { display: flex; flex-wrap: wrap; gap: 1.5rem 3rem; padding: 1rem 1.3rem; border-top: 1px solid var(--line); background: #e3e6e1; }
	.document-footer div { min-width: 8rem; }
	.document-footer span, .document-footer strong { display: block; }
	.document-footer span { margin-bottom: .25rem; color: #5f7376; font-size: .48rem; letter-spacing: .13em; }
	.document-footer strong { overflow-wrap: anywhere; color: #2a4246; font-size: .58rem; font-weight: 500; }
	.compact-brief { overflow: hidden; border: 1px solid #c7d0cc; background: #f7f7f2; font-family: 'IBM Plex Mono', monospace; color: #142c33; }
	.compact-brief header { display: flex; align-items: center; justify-content: space-between; gap: 1rem; padding: 1rem 1.2rem; background: #102830; color: #e7f0ed; }
	.compact-brief header p { margin: 0 0 .2rem; color: #8fd2c0; font-size: .5rem; letter-spacing: .14em; }
	.compact-brief header h3 { margin: 0; font: 600 1.3rem 'Barlow Condensed', sans-serif; text-transform: uppercase; }
	.compact-brief header > span { color: #91a6a6; font-size: .55rem; }
	.compact-summary { margin: 0; padding: 1.2rem; color: var(--body, #1a3339); font: 400 .88rem/1.65 'IBM Plex Sans', ui-sans-serif, system-ui, sans-serif; letter-spacing: -.01em; }
	.compact-findings { border-top: 1px solid #d2d8d4; }
	.compact-findings div { display: grid; grid-template-columns: 2.6rem 1fr; gap: .6rem; padding: .7rem 1.2rem; border-bottom: 1px solid #dce0dd; font: 400 .78rem/1.45 'IBM Plex Sans', ui-sans-serif, system-ui, sans-serif; }
	.compact-findings b { color: #397078; font: 600 .52rem 'IBM Plex Mono', monospace; }
	.compact-findings div[data-severity='warning'] b { color: #a56b08; }
	.compact-findings div[data-severity='critical'] b { color: #b43d34; }
	@media (max-width: 800px) { .brief-header { grid-template-columns: 1fr; } .risk-stamp { min-height: 12rem; border-top: 1px solid var(--line); border-left: 0; } .follow-up-grid { grid-template-columns: 1fr; } .recommendation-panel { border-top: 1px solid var(--line); border-left: 0; } }
	@media (max-width: 680px) { .document-rail { align-items: stretch; padding-left: .8rem; } .rail-metadata { flex-direction: column; align-items: flex-start; justify-content: center; gap: .25rem; } .rail-metadata span:nth-child(2) { display: none; } }
	@media (max-width: 560px) { .pdf-download small { display: none; } .pdf-download svg { grid-row: auto; } .finding { grid-template-columns: 2.8rem 1fr; } .finding-main { padding: 1.1rem; } .finding-meta time { width: 100%; margin-left: 0; } .evidence-block ul { grid-template-columns: 1fr; } .summary-block { padding: 2rem 1.3rem; } .findings-section { padding: 1.3rem; } }
</style>
