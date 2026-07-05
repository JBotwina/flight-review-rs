import { writable } from 'svelte/store';
import { browser } from '$app/environment';

function createThemeStore() {
	const initial = browser
		? window.localStorage.getItem('theme') === 'dark' ||
			(!window.localStorage.getItem('theme') &&
				window.matchMedia('(prefers-color-scheme: dark)').matches)
		: false;
	const { subscribe, set, update } = writable(initial);
	return {
		subscribe,
		toggle: () =>
			update((dark) => {
				const next = !dark;
				if (browser) {
					window.localStorage.setItem('theme', next ? 'dark' : 'light');
					document.documentElement.classList.toggle('dark', next);
				}
				return next;
			}),
		set: (dark: boolean) => {
			if (browser) {
				window.localStorage.setItem('theme', dark ? 'dark' : 'light');
				document.documentElement.classList.toggle('dark', dark);
			}
			set(dark);
		}
	};
}
export const darkMode = createThemeStore();
