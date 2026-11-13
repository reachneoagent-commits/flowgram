import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { writable } from 'svelte/store';
import {
	Client,
	type HumanizerDownloadDone,
	type HumanizerDownloadProgress,
	type HumanizerStatus,
} from './client';

export interface HumanizerView {
	status: HumanizerStatus | null;
	loading: boolean;
	progress: HumanizerDownloadProgress | null;
	error: string;
}

/** Shared by onboarding and Settings so both always show the same state. */
export const humanizer = writable<HumanizerView>({
	status: null,
	loading: true,
	progress: null,
	error: '',
});

export async function refreshHumanizer() {
	try {
		const status = await Client.humanizerStatus();
		humanizer.update((view) => ({ ...view, status, loading: false }));
	} catch (error) {
		humanizer.update((view) => ({
			...view,
			loading: false,
			error: `Couldn't check this computer: ${error}`,
		}));
	}
}

/** Starts listening for download events. Call the returned function to stop. */
export async function watchHumanizer(): Promise<UnlistenFn> {
	const stops = await Promise.all([
		listen<HumanizerDownloadProgress>('humanizer-download', ({ payload }) =>
			humanizer.update((view) => ({ ...view, progress: payload })),
		),
		listen<HumanizerDownloadDone>('humanizer-download-done', ({ payload }) => {
			humanizer.update((view) => ({
				...view,
				progress: null,
				error: payload.error ?? '',
			}));
			void refreshHumanizer();
		}),
	]);
	return () => {
		for (const stop of stops) stop();
	};
}

export async function startHumanizerDownload(tier: string) {
	humanizer.update((view) => ({ ...view, error: '' }));
	try {
		await Client.humanizerDownload(tier);
		await refreshHumanizer();
	} catch (error) {
		humanizer.update((view) => ({ ...view, error: `${error}` }));
	}
}

export async function cancelHumanizerDownload() {
	try {
		await Client.humanizerCancelDownload();
	} catch (error) {
		humanizer.update((view) => ({ ...view, error: `${error}` }));
	}
}

export function formatGb(bytes: number): string {
	return `${(bytes / 1e9).toFixed(1).replace(/\.0$/, '')} GB`;
}
