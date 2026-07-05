import { ApiError } from '$lib/api-error';

type RequestConfig = {
	url: string;
	method: string;
	params?: Record<string, unknown>;
	data?: BodyInit | object;
	headers?: HeadersInit;
	signal?: AbortSignal;
};

function appendQueryParams(url: string, params?: Record<string, unknown>) {
	if (!params) return url;
	const search = new URLSearchParams();
	for (const [key, value] of Object.entries(params)) {
		if (value === undefined || value === null || value === '') continue;
		search.set(key, String(value));
	}
	const query = search.toString();
	if (!query) return url;
	return `${url}${url.includes('?') ? '&' : '?'}${query}`;
}

export const customFetch = async <T>(
	config: RequestConfig,
	options?: RequestInit
): Promise<T> => {
	const headers = new Headers(options?.headers ?? config.headers);
	let body = options?.body ?? (config.data as BodyInit | undefined);
	if (body && typeof body === 'object' && !(body instanceof FormData) && !(body instanceof Blob)) {
		headers.set('Content-Type', headers.get('Content-Type') ?? 'application/json');
		body = JSON.stringify(body);
	}

	const res = await fetch(appendQueryParams(config.url, config.params), {
		...options,
		method: config.method,
		headers,
		body,
		signal: config.signal ?? options?.signal,
	});
	if (!res.ok) {
		const body = await res.text().catch(() => '');
		throw new ApiError(res.status, body || res.statusText);
	}
	if (res.status === 204) return undefined as T;
	return res.json() as Promise<T>;
};
