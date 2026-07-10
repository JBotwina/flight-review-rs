<script lang="ts">
	function goBack() {
		if (history.length > 1) history.back();
	}
</script>

<svelte:head>
	<title>404 · Not found</title>
	<meta name="robots" content="noindex, nofollow" />
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
	<link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&display=swap" rel="stylesheet" />
</svelte:head>

<div class="not-found">
	<div class="noise" aria-hidden="true"></div>
	<main>
		<p class="code">404</p>
		<h1>Nothing here.</h1>
		<p class="message">The requested resource could not be found.</p>
		<button type="button" onclick={goBack}>Go back</button>
	</main>
	<p class="request">ERR_ROUTE_NOT_FOUND</p>
</div>

<style>
	:global(body) { margin: 0; background: #090d10; }
	.not-found { position: relative; min-height: 100vh; display: grid; place-items: center; overflow: hidden; background: radial-gradient(circle at 50% 42%, #161d21 0, #0b1013 43%, #070a0c 100%); color: #c9d1d2; font-family: 'IBM Plex Mono', monospace; }
	.not-found::before, .not-found::after { content: ''; position: absolute; border: 1px solid rgba(148,165,166,.08); border-radius: 50%; }
	.not-found::before { width: 42rem; height: 42rem; }
	.not-found::after { width: 26rem; height: 26rem; }
	.noise { position: absolute; inset: 0; opacity: .08; background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 160 160' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='.9' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='.7'/%3E%3C/svg%3E"); pointer-events: none; }
	main { position: relative; z-index: 1; width: min(32rem, calc(100vw - 3rem)); text-align: center; }
	.code { margin: 0 0 1.25rem; color: #637175; font-size: .72rem; letter-spacing: .3em; }
	h1 { margin: 0; color: #edf2f1; font-size: clamp(2.5rem, 6vw, 4.5rem); font-weight: 400; letter-spacing: -.055em; }
	.message { margin: 1.25rem 0 2rem; color: #77868a; font-size: .78rem; line-height: 1.7; }
	button { border: 1px solid #344044; padding: .7rem 1.05rem; background: rgba(13,19,22,.55); color: #a8b3b4; font: 500 .68rem 'IBM Plex Mono', monospace; letter-spacing: .08em; cursor: pointer; transition: border-color .2s, color .2s; }
	button:hover { border-color: #718083; color: #f1f4f3; }
	.request { position: absolute; right: 1.5rem; bottom: 1.25rem; margin: 0; color: #394347; font-size: .56rem; letter-spacing: .14em; }
</style>
