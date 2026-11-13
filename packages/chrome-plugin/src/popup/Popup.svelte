<script lang="ts">
import { faArrowLeft } from '@fortawesome/free-solid-svg-icons';
import { Button, Link } from 'components';
import { fly } from 'svelte/transition';
import Fa from 'svelte-fa';
import logo from '/logo.png';
import { main, type PopupState } from '../PopupState';
import Main from './Main.svelte';
import Onboarding from './Onboarding.svelte';

let popupState: PopupState = $state({ page: 'main' });

let version = `v${chrome.runtime.getManifest().version}`;

$effect(() => {
	chrome.storage.local.get({ popupState: { page: 'onboarding' } }).then((result) => {
		popupState = result.popupState;
	});
});

$effect(() => {
	chrome.storage.local.set({ popupState: $state.snapshot(popupState) });
});

function openSettings() {
	chrome.runtime?.openOptionsPage?.();
}

// Per-page heights: compact for the main view, taller for forms.
const PAGE_HEIGHTS: Record<string, number> = { main: 410, 'report-error': 580 };
let heightPx = $derived(PAGE_HEIGHTS[popupState.page] ?? 500);
const WIDTH_PX = 340;

// Browser popups size themselves to the document, and stale CSS (e.g. #app min-h-screen) can keep
// them at the larger size. Setting the size explicitly forces the popup to resize.
$effect(() => {
	const app = document.getElementById('app'); // global CSS gives #app min-h-screen
	for (const el of [document.documentElement, document.body, app]) {
		if (!el) continue;
		el.style.width = `${WIDTH_PX}px`;
		el.style.height = `${heightPx}px`;
		el.style.minHeight = '0';
		el.style.margin = '0';
		el.style.overflow = 'hidden';
	}
});
</script>

<div
    style="width: {WIDTH_PX}px; height: {heightPx}px;"
    class="font-sans flex flex-col select-none overflow-hidden rounded-xl border border-gray-200 dark:border-slate-800 dark:text-slate-100"
>
    <header
        class="flex shrink-0 flex-row items-center justify-between gap-2 border-b border-gray-100 px-4 py-3 dark:border-slate-800"
    >
        <div class="flex flex-row items-center gap-2.5">
            <img src={logo} alt="Gramflow logo" class="h-6 w-6 rounded-md" />
            <span class="text-sm font-semibold tracking-tight">Gramflow</span>
        </div>

        {#if popupState.page != 'main'}
            <Button
                class="bg-primary! text-black! hover:bg-primary/85!"
                on:click={() => {
                    popupState = main();
                }}
            >
                <Fa icon={faArrowLeft} />
            </Button>
        {:else}
            <span class="px-1.5 py-0.5 font-mono text-xs text-gray-500 dark:text-slate-400">{version}</span>
        {/if}
    </header>

    <!-- Flex container ensuring children take up full available height -->
    <div class="relative flex min-h-0 flex-1 flex-col overflow-hidden">
        {#if popupState.page == 'onboarding'}
            <div class="flex h-full flex-col" in:fly={{ y: 12, duration: 250 }}>
                <Onboarding
                    onConfirm={() => {
                        popupState = main();
                    }}
                />
            </div>
        {:else if popupState.page == 'main'}
            <div class="flex h-full flex-col" in:fly={{ y: 12, duration: 250 }}>
                <Main />
            </div>
        {/if}
    </div>

    <footer class="flex items-center justify-center gap-6 px-3 py-2 text-sm border-t border-gray-100 rounded-b-lg bg-white/60 dark:border-slate-800 dark:bg-slate-900/70 dark:text-slate-100">
        <Link on:click={openSettings}>Settings</Link>
    </footer>
</div>