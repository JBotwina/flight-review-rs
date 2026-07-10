<section class="working-state" role="status" aria-live="polite" aria-busy="true">
	<div class="working-content">
		<div class="spinner" aria-hidden="true">
			<span></span>
		</div>
		<p>AI / WORKING</p>
		<h3>Correlating flight evidence</h3>
	</div>
</section>

<style>
	.working-state {
		min-height: 22rem;
		display: grid;
		place-items: center;
		align-content: center;
		margin-top: 1rem;
		border: 1px solid #cbd3cf;
		padding: 3rem 1.5rem;
		background: #f7f7f2;
		color: #10262d;
		text-align: center;
	}

	.working-content {
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	.spinner {
		position: relative;
		width: 4.25rem;
		height: 4.25rem;
		border: 2px solid rgba(23, 105, 115, .16);
		border-radius: 50%;
	}

	.spinner::before {
		content: '';
		position: absolute;
		inset: -2px;
		border: 2px solid transparent;
		border-top-color: #176973;
		border-right-color: #c8ef4b;
		border-radius: 50%;
		animation: spin 1.35s linear infinite;
	}

	.spinner::after {
		content: '';
		position: absolute;
		inset: .65rem;
		border: 1px dashed rgba(23, 105, 115, .45);
		border-radius: 50%;
		animation: spin-reverse 4s linear infinite;
	}

	.spinner span {
		position: absolute;
		top: 50%;
		left: 50%;
		z-index: 1;
		width: .55rem;
		height: .55rem;
		border: 1px solid #176973;
		background: #c8ef4b;
		transform: translate(-50%, -50%) rotate(45deg);
	}

	p {
		margin: 1.35rem 0 .45rem;
		color: #5f7b7e;
		font: 600 .58rem 'IBM Plex Mono', monospace;
		letter-spacing: .2em;
	}

	h3 {
		margin: 0;
		font: 600 clamp(1.7rem, 4vw, 2.2rem) 'Barlow Condensed', sans-serif;
		letter-spacing: -.01em;
		text-transform: uppercase;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	@keyframes spin-reverse {
		to { transform: rotate(-360deg); }
	}

	@media (prefers-reduced-motion: reduce) {
		.spinner::before { animation-duration: 2.7s; }
		.spinner::after { animation-duration: 8s; }
	}
</style>
