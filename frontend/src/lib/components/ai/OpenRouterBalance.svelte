<script lang="ts">
	import type { AiBalanceResponse } from '$lib/types';

	let { balance, compact = false, variant = 'default' }: { balance: AiBalanceResponse; compact?: boolean; variant?: 'default' | 'console' } = $props();

	function dollars(value: number): string {
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: 'USD',
			minimumFractionDigits: 2,
			maximumFractionDigits: value < 1 ? 4 : 2,
		}).format(value);
	}
</script>

{#if balance.enabled}
	<div
		class="inline-flex flex-wrap items-center gap-x-2 gap-y-0.5 border px-2.5 py-1.5 text-xs {variant === 'console'
			? 'rounded-sm border-lime-300/25 bg-lime-300/8 font-mono text-lime-200'
			: 'rounded-md border-emerald-200 bg-emerald-50 text-emerald-900'}"
		title="OpenRouter API-key usage. Account-wide credit balance requires a management key."
	>
		<span class="font-bold">OpenRouter</span>
		{#if balance.limit_remaining !== null}
			<span><strong>{dollars(balance.limit_remaining)}</strong> remaining</span>
			{#if !compact && balance.limit !== null}
				<span class={variant === 'console' ? 'text-lime-200/55' : 'text-emerald-700'}>of {dollars(balance.limit)}</span>
			{/if}
		{:else}
			<span>No key spending limit</span>
		{/if}
		{#if !compact}
			<span class={variant === 'console' ? 'text-lime-200/55' : 'text-emerald-700'}>{dollars(balance.usage)} used</span>
		{/if}
	</div>
{/if}
