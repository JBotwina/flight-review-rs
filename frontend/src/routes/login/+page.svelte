<script lang="ts">
	import { loginWithPassword } from '$lib/api';

	let password = $state('');
	let submitting = $state(false);
	let revealPassword = $state(false);
	let error = $state('');

	async function handleSubmit(event: SubmitEvent) {
		event.preventDefault();
		if (!password || submitting) return;

		submitting = true;
		error = '';
		try {
			await loginWithPassword(password);
			window.location.replace('/');
		} catch {
			error = 'Access code not recognized. Check the briefing and try again.';
			password = '';
		} finally {
			submitting = false;
		}
	}
</script>

<svelte:head>
	<title>Pilot access · PX4 Flight Review</title>
	<meta name="robots" content="noindex, nofollow" />
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
	<link href="https://fonts.googleapis.com/css2?family=Barlow+Condensed:wght@500;600;700&family=IBM+Plex+Mono:wght@400;500;600&display=swap" rel="stylesheet" />
</svelte:head>

<div class="access-shell">
	<div class="grid-field" aria-hidden="true"></div>
	<div class="scan-line" aria-hidden="true"></div>

	<section class="briefing" aria-label="PX4 Flight Review pilot program">
		<div class="brand-lockup">
			<div class="mark" aria-hidden="true">
				<svg viewBox="0 0 64 64" fill="none">
					<circle cx="32" cy="32" r="5" stroke="currentColor" stroke-width="2" />
					<path d="M28 29 16 17M36 29l12-12M28 35 16 47M36 35l12 12" stroke="currentColor" stroke-width="2" />
					<circle cx="12" cy="13" r="8" stroke="currentColor" stroke-width="2" />
					<circle cx="52" cy="13" r="8" stroke="currentColor" stroke-width="2" />
					<circle cx="12" cy="51" r="8" stroke="currentColor" stroke-width="2" />
					<circle cx="52" cy="51" r="8" stroke="currentColor" stroke-width="2" />
				</svg>
			</div>
			<div>
				<p class="eyebrow">PX4 community pilot</p>
				<p class="wordmark">Flight Review</p>
			</div>
		</div>

		<div class="mission-copy">
			<p class="mission-index">FR / AI–01</p>
			<h1>Read the flight.<br /><span>Question the machine.</span></h1>
			<p class="lede">A private evaluation workspace for PX4 engineers testing evidence-backed AI analysis against real ULog data.</p>
		</div>

		<div class="telemetry-strip" aria-label="Program characteristics">
			<div><span>INPUT</span><strong>ULog</strong></div>
			<div><span>PIPELINE</span><strong>Rust + Parquet</strong></div>
			<div><span>ACCESS</span><strong>Briefed crew</strong></div>
		</div>
	</section>

	<section class="access-panel">
		<div class="panel-frame">
			<div class="panel-status">
				<span class="status-light"></span>
				<span>Restricted test environment</span>
				<span class="station">STN 47</span>
			</div>

			<div class="panel-body">
				<p class="sequence">01 — AUTHENTICATE</p>
				<h2>Enter pilot access code</h2>
				<p class="instructions">Use the shared code from your PX4 Flight Review briefing. Your session stays unlocked on this device for seven days.</p>

				<form onsubmit={handleSubmit}>
					<label for="access-password">Access code</label>
					<div class:error class="password-field">
						<input
							id="access-password"
							type={revealPassword ? 'text' : 'password'}
							bind:value={password}
							autocomplete="current-password"
							autocapitalize="none"
							spellcheck="false"
							placeholder="••••••••••••••••"
							disabled={submitting}
							required
						/>
						<button type="button" class="reveal" onclick={() => (revealPassword = !revealPassword)} aria-label={revealPassword ? 'Hide access code' : 'Show access code'}>
							{revealPassword ? 'HIDE' : 'SHOW'}
						</button>
					</div>
					{#if error}
						<p class="error-message" role="alert"><span>!</span>{error}</p>
					{/if}
					<button class="submit" type="submit" disabled={submitting || !password}>
						<span>{submitting ? 'Validating access…' : 'Unlock flight review'}</span>
						<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12h13m-5-5 5 5-5 5" /></svg>
					</button>
				</form>
			</div>

			<div class="panel-footer">
				<span>AUTHORIZED EVALUATION ONLY</span>
				<span>PX4 / FR2</span>
			</div>
		</div>
	</section>
</div>

<style>
	:global(body) { margin: 0; background: #07131d; }
	.access-shell { --ink: #dce9e6; --muted: #81989a; --lime: #b8e43d; --line: rgba(177, 212, 208, .16); position: relative; min-height: 100vh; overflow: hidden; display: grid; grid-template-columns: minmax(0, 1.15fr) minmax(390px, .85fr); background: #07131d; color: var(--ink); font-family: 'IBM Plex Mono', monospace; }
	.grid-field { position: absolute; inset: 0; opacity: .28; background-image: linear-gradient(var(--line) 1px, transparent 1px), linear-gradient(90deg, var(--line) 1px, transparent 1px); background-size: 52px 52px; mask-image: linear-gradient(90deg, black, transparent 78%); }
	.grid-field::after { content: ''; position: absolute; width: 48vw; height: 48vw; left: -12vw; bottom: -22vw; border: 1px solid rgba(184, 228, 61, .25); border-radius: 50%; box-shadow: 0 0 0 7vw rgba(184,228,61,.035), 0 0 0 14vw rgba(184,228,61,.025), 0 0 0 21vw rgba(184,228,61,.018); }
	.scan-line { position: absolute; z-index: 3; left: 0; right: 0; height: 1px; background: linear-gradient(90deg, transparent, rgba(184,228,61,.38), transparent); animation: scan 7s linear infinite; pointer-events: none; }
	.briefing { position: relative; z-index: 1; min-height: 100vh; padding: clamp(2rem, 5vw, 5.5rem); display: flex; flex-direction: column; justify-content: space-between; border-right: 1px solid var(--line); }
	.brand-lockup { display: flex; align-items: center; gap: 1rem; animation: enter .7s ease-out both; }
	.mark { width: 3.6rem; height: 3.6rem; color: var(--lime); }
	.mark svg { width: 100%; height: 100%; }
	.eyebrow, .sequence { margin: 0 0 .28rem; color: var(--lime); font-size: .68rem; font-weight: 600; letter-spacing: .2em; text-transform: uppercase; }
	.wordmark { margin: 0; font-family: 'Barlow Condensed', sans-serif; font-size: 1.65rem; font-weight: 700; letter-spacing: .03em; text-transform: uppercase; }
	.mission-copy { max-width: 49rem; padding: 6rem 0 4rem; animation: enter .8s .12s ease-out both; }
	.mission-index { margin: 0 0 1rem; color: var(--muted); font-size: .72rem; letter-spacing: .16em; }
	h1 { margin: 0; font-family: 'Barlow Condensed', sans-serif; font-size: clamp(4rem, 7vw, 7.4rem); font-weight: 600; line-height: .86; letter-spacing: -.035em; text-transform: uppercase; }
	h1 span { color: transparent; -webkit-text-stroke: 1px rgba(220,233,230,.72); }
	.lede { max-width: 39rem; margin: 2.2rem 0 0; color: #9db0b0; font-size: clamp(.85rem, 1.1vw, 1rem); line-height: 1.75; }
	.telemetry-strip { display: grid; grid-template-columns: repeat(3, 1fr); border-top: 1px solid var(--line); animation: enter .8s .24s ease-out both; }
	.telemetry-strip div { padding: 1.1rem 1rem 0 0; }
	.telemetry-strip span, .telemetry-strip strong { display: block; }
	.telemetry-strip span { margin-bottom: .4rem; color: #587174; font-size: .58rem; letter-spacing: .16em; }
	.telemetry-strip strong { font-size: .74rem; font-weight: 500; }
	.access-panel { position: relative; z-index: 2; min-height: 100vh; padding: clamp(1rem, 4vw, 4rem); display: grid; place-items: center; background: radial-gradient(circle at 55% 35%, rgba(28,62,70,.55), transparent 46%), rgba(4, 13, 20, .72); }
	.panel-frame { width: min(100%, 31rem); border: 1px solid rgba(184,228,61,.28); background: rgba(7,19,29,.88); box-shadow: 0 28px 90px rgba(0,0,0,.45), inset 0 0 60px rgba(73,113,108,.04); clip-path: polygon(0 0, calc(100% - 1.1rem) 0, 100% 1.1rem, 100% 100%, 1.1rem 100%, 0 calc(100% - 1.1rem)); animation: panel-in .7s .15s cubic-bezier(.16,1,.3,1) both; }
	.panel-status, .panel-footer { display: flex; align-items: center; gap: .65rem; padding: .85rem 1rem; color: #789092; font-size: .58rem; letter-spacing: .13em; text-transform: uppercase; border-bottom: 1px solid var(--line); }
	.status-light { width: .46rem; height: .46rem; border-radius: 50%; background: var(--lime); box-shadow: 0 0 15px rgba(184,228,61,.75); animation: pulse 2.4s ease-in-out infinite; }
	.station { margin-left: auto; }
	.panel-body { padding: clamp(1.7rem, 4vw, 3.2rem); }
	h2 { margin: .7rem 0 0; font-family: 'Barlow Condensed', sans-serif; font-size: clamp(2.4rem, 4vw, 3.35rem); font-weight: 600; line-height: .95; text-transform: uppercase; }
	.instructions { margin: 1.15rem 0 2rem; color: var(--muted); font-size: .74rem; line-height: 1.7; }
	form label { display: block; margin-bottom: .6rem; color: #9fb3b3; font-size: .64rem; font-weight: 600; letter-spacing: .12em; text-transform: uppercase; }
	.password-field { display: flex; align-items: center; border: 1px solid #365055; background: rgba(2,10,15,.68); transition: border-color .2s, box-shadow .2s; }
	.password-field:focus-within { border-color: var(--lime); box-shadow: 0 0 0 3px rgba(184,228,61,.09); }
	.password-field.error { border-color: #ef715e; }
	input { min-width: 0; flex: 1; border: 0; outline: 0; padding: 1rem; background: transparent; color: #f0f6f3; font: 500 .86rem 'IBM Plex Mono', monospace; letter-spacing: .08em; }
	input::placeholder { color: #40575b; }
	.reveal { align-self: stretch; border: 0; border-left: 1px solid #253d42; padding: 0 .9rem; background: transparent; color: #81989a; font: 600 .58rem 'IBM Plex Mono', monospace; letter-spacing: .12em; cursor: pointer; }
	.reveal:hover { color: var(--lime); }
	.error-message { display: flex; gap: .55rem; margin: .75rem 0 0; color: #ef8b79; font-size: .67rem; line-height: 1.5; }
	.error-message span { font-weight: 700; }
	.submit { width: 100%; margin-top: 1rem; display: flex; align-items: center; justify-content: space-between; border: 1px solid var(--lime); padding: .95rem 1rem; background: var(--lime); color: #07131d; font: 600 .69rem 'IBM Plex Mono', monospace; letter-spacing: .1em; text-transform: uppercase; cursor: pointer; transition: background .2s, color .2s, transform .2s; }
	.submit svg { width: 1.2rem; fill: none; stroke: currentColor; stroke-width: 1.8; }
	.submit:hover:not(:disabled) { background: transparent; color: var(--lime); transform: translateY(-1px); }
	.submit:disabled { cursor: not-allowed; opacity: .45; }
	.panel-footer { justify-content: space-between; border-top: 1px solid var(--line); border-bottom: 0; }
	@keyframes enter { from { opacity: 0; transform: translateY(18px); } to { opacity: 1; transform: none; } }
	@keyframes panel-in { from { opacity: 0; transform: translateX(24px) scale(.985); } to { opacity: 1; transform: none; } }
	@keyframes pulse { 50% { opacity: .45; box-shadow: 0 0 5px rgba(184,228,61,.35); } }
	@keyframes scan { from { top: -2%; } to { top: 102%; } }
	@media (max-width: 900px) { .access-shell { grid-template-columns: 1fr; } .briefing { min-height: auto; padding-bottom: 1.5rem; border-right: 0; } .mission-copy { padding: 5rem 0 3rem; } .telemetry-strip { display: none; } .access-panel { min-height: auto; padding: 1.5rem 1rem 3rem; } }
	@media (max-width: 520px) { .briefing { padding: 1.3rem; } .mission-copy { padding: 4rem 0 1.5rem; } h1 { font-size: 3.5rem; } .lede { font-size: .76rem; } .panel-body { padding: 1.5rem; } }
	@media (prefers-reduced-motion: reduce) { *, *::before, *::after { animation-duration: .01ms !important; animation-iteration-count: 1 !important; } }
</style>
