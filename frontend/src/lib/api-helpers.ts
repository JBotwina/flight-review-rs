import type { FlightMetadata } from '$lib/types';
import { customFetch } from '$lib/generated/custom-fetch';

export function pageToOffset(page: number, limit: number) {
	return (page - 1) * limit;
}

export async function getMetadata(id: string): Promise<FlightMetadata> {
	return customFetch<FlightMetadata>({
		url: `/api/logs/${id}/data/metadata.json`,
		method: 'GET',
	});
}

export function getMetadataQueryKey(id: string) {
	return [`/api/logs/${id}/data/metadata.json`] as const;
}
