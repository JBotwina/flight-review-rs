<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { createQuery, keepPreviousData } from '@tanstack/svelte-query';
	import { toStore } from 'svelte/store';
	import { getGetStatsQueryOptions } from '$lib/generated/endpoints/stats/stats';
	import type { GetStatsParams } from '$lib/generated/models';
	import type { StatsFilters, StatsDataPoint } from '$lib/types';
	import StatsFilterPanel from '$lib/components/stats/StatsFilterPanel.svelte';
	import KpiCards from '$lib/components/stats/KpiCards.svelte';
	import HardwareBars from '$lib/components/stats/HardwareBars.svelte';
	import VehicleTypeDonut from '$lib/components/stats/VehicleTypeDonut.svelte';
	import DurationHistogram from '$lib/components/stats/DurationHistogram.svelte';
	import AirframesTable from '$lib/components/stats/AirframesTable.svelte';

	// URL-driven state
	let period = $derived(page.url.searchParams.get('period') ?? 'all');
	let filters = $derived.by((): StatsFilters => ({
		vehicleType: page.url.searchParams.get('vehicle_type') || undefined,
		verHw: page.url.searchParams.get('ver_hw') || undefined,
		source: page.url.searchParams.get('source') || undefined,
	}));

	function updateUrl(updates: Record<string, string | undefined>) {
		const params = new URLSearchParams(page.url.searchParams);
		for (const [key, value] of Object.entries(updates)) {
			if (value === undefined || value === '') {
				params.delete(key);
			} else {
				params.set(key, value);
			}
		}
		const search = params.toString();
		goto(`/stats${search ? `?${search}` : ''}`, { replaceState: true, keepFocus: true });
	}

	function handlePeriodChange(newPeriod: string) {
		updateUrl({ period: newPeriod === 'all' ? undefined : newPeriod });
	}

	function handleFiltersChange(newFilters: StatsFilters) {
		updateUrl({
			vehicle_type: newFilters.vehicleType,
			ver_hw: newFilters.verHw,
			source: newFilters.source,
		});
	}

	function handleHardwareClick(hardware: string) {
		updateUrl({ ver_hw: hardware });
	}

	function handleAirframeClick(airframe: string) {
		updateUrl({ vehicle_type: airframe });
	}

	function buildParams(groupBy: string, limit?: number): GetStatsParams {
		return {
			group_by: groupBy,
			period: period === 'all' ? undefined : period,
			limit,
			vehicle_type: filters.vehicleType,
			ver_hw: filters.verHw,
			source: filters.source,
		};
	}

	const queryOptions = { placeholderData: keepPreviousData };
	const hwQuery = createQuery(
		toStore(() => getGetStatsQueryOptions(buildParams('ver_hw', 15), { query: queryOptions }))
	);
	const vehicleQuery = createQuery(
		toStore(() => getGetStatsQueryOptions(buildParams('vehicle_type'), { query: queryOptions }))
	);
	const durationQuery = createQuery(
		toStore(() => getGetStatsQueryOptions(buildParams('mission_type'), { query: queryOptions }))
	);
	const airframeQuery = createQuery(
		toStore(() => getGetStatsQueryOptions(buildParams('sys_name', 25), { query: queryOptions }))
	);

	let hwData = $derived<StatsDataPoint[]>($hwQuery.data?.data ?? []);
	let vehicleData = $derived<StatsDataPoint[]>($vehicleQuery.data?.data ?? []);
	let durationData = $derived<StatsDataPoint[]>(
		$durationQuery.isError ? [] : ($durationQuery.data?.data ?? [])
	);
	let airframeData = $derived<StatsDataPoint[]>($airframeQuery.data?.data ?? []);

	let totalLogs = $derived(hwData.reduce((s, d) => s + d.count, 0));
	let flightHours = $derived(hwData.reduce((s, d) => s + (d.total_flight_hours ?? 0), 0));
	let uniqueVehicles = $derived(hwData.length);
	let todayUploads = $derived(airframeData.reduce((s, d) => s + d.count, 0));

	let loadingHw = $derived($hwQuery.isPending);
	let loadingVehicle = $derived($vehicleQuery.isPending);
	let loadingDuration = $derived($durationQuery.isPending);
	let loadingAirframe = $derived($airframeQuery.isPending);
	let error = $derived(
		$hwQuery.error || $vehicleQuery.error || $airframeQuery.error
			? 'Failed to load statistics'
			: null
	);
</script>

<svelte:head>
	<title>Statistics - Flight Review</title>
</svelte:head>

<div class="px-4 sm:px-6 lg:px-8 py-8">
	<!-- Header -->
	<div class="mb-6">
		<h1 class="text-base font-semibold text-gray-900 dark:text-gray-100">Statistics</h1>
		<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">Flight log analytics and trends</p>
	</div>

	<!-- Filter panel -->
	<div class="mb-6">
		<StatsFilterPanel
			{period}
			{filters}
			onPeriodChange={handlePeriodChange}
			onFiltersChange={handleFiltersChange}
		/>
	</div>

	<!-- Error banner -->
	{#if error}
		<div class="rounded-md bg-red-50 p-4 mb-6 dark:bg-red-900/20">
			<div class="flex">
				<div class="shrink-0">
					<svg class="size-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
						<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.28 7.22a.75.75 0 00-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 101.06 1.06L10 11.06l1.72 1.72a.75.75 0 101.06-1.06L11.06 10l1.72-1.72a.75.75 0 00-1.06-1.06L10 8.94 8.28 7.22z" clip-rule="evenodd" />
					</svg>
				</div>
				<div class="ml-3">
					<p class="text-sm text-red-700 dark:text-red-400">{error}</p>
				</div>
			</div>
		</div>
	{/if}

	<!-- KPI Cards -->
	<div class="mb-6">
		<KpiCards
			{totalLogs}
			{flightHours}
			{uniqueVehicles}
			{todayUploads}
			loading={loadingHw}
		/>
	</div>

	<!-- Charts grid: single column on mobile, 2 columns on lg+ -->
	<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
		<HardwareBars data={hwData} loading={loadingHw} onBarClick={handleHardwareClick} />
		<VehicleTypeDonut data={vehicleData} loading={loadingVehicle} />
		<DurationHistogram data={durationData} loading={loadingDuration} />
		<AirframesTable data={airframeData} loading={loadingAirframe} onAirframeClick={handleAirframeClick} />
	</div>
</div>
