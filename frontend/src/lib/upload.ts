import { ApiError } from '$lib/api-error';
import type { UploadResponse } from '$lib/generated/models';
import type { UploadOptions } from '$lib/types';

export function uploadLog(
	file: File,
	opts: UploadOptions,
	onProgress?: (pct: number) => void
): { promise: Promise<UploadResponse>; abort: () => void } {
	const xhr = new XMLHttpRequest();
	const promise = new Promise<UploadResponse>((resolve, reject) => {
		const form = new FormData();
		form.append('file', file);
		if (opts.description) form.append('description', opts.description);
		if (opts.isPublic) form.append('is_public', 'true');
		if (opts.windSpeed) form.append('wind_speed', opts.windSpeed);
		if (opts.rating != null) form.append('rating', String(opts.rating));
		if (opts.feedback) form.append('feedback', opts.feedback);
		if (opts.videoUrl) form.append('video_url', opts.videoUrl);
		if (opts.source) form.append('source', opts.source);
		if (opts.pilotName) form.append('pilot_name', opts.pilotName);
		if (opts.vehicleName) form.append('vehicle_name', opts.vehicleName);
		if (opts.tags) form.append('tags', opts.tags);
		if (opts.locationName) form.append('location_name', opts.locationName);
		if (opts.missionType) form.append('mission_type', opts.missionType);

		xhr.upload.onprogress = (e) => {
			if (e.lengthComputable && onProgress) onProgress((e.loaded / e.total) * 100);
		};
		xhr.onload = () => {
			if (xhr.status >= 200 && xhr.status < 300) {
				resolve(JSON.parse(xhr.responseText));
			} else {
				reject(new ApiError(xhr.status, xhr.responseText || xhr.statusText));
			}
		};
		xhr.onerror = () => reject(new Error('Network error'));
		xhr.open('POST', '/api/upload');
		xhr.send(form);
	});
	return { promise, abort: () => xhr.abort() };
}
