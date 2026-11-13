<script lang="ts">
import { Button } from 'components';
import generateGreeting from '../generateGreeting';
import ProtocolClient from '../ProtocolClient';

// UI State Flags
let loaded = $state(false);

let enabled = $state(true);
let domain = $state('');
let site = $derived(domain || 'this page');

let installDate: Date | null = $state(null);
let hasBeenReviewed: boolean | null = $state(null);

const isFirefox = isFirefoxExtension();

// Fetch all initial data concurrently before showing the UI to prevent layout flashing
Promise.all([
	getCurrentTabDomain().then(async (d) => {
		domain = d ?? '';
		enabled = await ProtocolClient.getDomainEnabled(domain);
	}),
	!isFirefox
		? ProtocolClient.getInstalledOn().then((d) => {
				if (d != null) installDate = new Date(d);
			})
		: Promise.resolve(),
	!isFirefox
		? ProtocolClient.getReviewed().then((r) => {
				hasBeenReviewed = r;
			})
		: Promise.resolve(),
]).then(() => {
	loaded = true;
});

/**
 * Returns the registrable domain (e.g.  "example.com") of the
 * tab that the user had open when they clicked the extension icon.
 * If the URL is unavailable (about:blank, chrome://…) it resolves to undefined.
 */
export async function getCurrentTabDomain(): Promise<string | undefined> {
	const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });

	if (!tab?.url) return undefined;

	try {
		const { hostname } = new URL(tab.url);
		return hostname.replace(/^www\./, '');
	} catch {
		return undefined;
	}
}

function toggleDomainEnabled() {
	enabled = !enabled;
	ProtocolClient.setDomainEnabled(domain, enabled);
}

function isFirefoxExtension(): boolean {
	try {
		return new URL(chrome.runtime.getURL('')).protocol === 'moz-extension:';
	} catch {
		return false;
	}
}

/** Get the number of days since a given Date. */
function daysSince(date: Date): number {
	let now = Date.now();
	let then = date.getTime();

	let msDiff = now - then;
	return msDiff / 86400000;
}
</script>

<main class="flex h-full flex-col justify-between">
    {#if loaded}
        <section
            class="flex flex-1 flex-col items-center justify-center px-5 py-6 transition-colors"
        >
            <!-- Row Layout: power button left, greeting + status right -->
            <div class="flex w-full flex-row items-center gap-5">

                <!-- 1. Power Button -->
                <Button
                    size="lg"
                    class="rounded-full! aspect-square h-24 w-24 shrink-0 p-0 shadow-lg transition-colors flex! flex-row items-center justify-center"
                    color={enabled ? 'var(--color-primary)' : 'var(--color-cream-50)'}
                    on:click={toggleDomainEnabled}
                    aria-pressed={enabled}
                    aria-label={enabled ? `Turn Gramflow off on ${site}` : `Turn Gramflow on for ${site}`}
                    title={enabled ? 'Click to pause Gramflow here' : 'Click to enable Gramflow here'}
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-9 w-9"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        stroke-width="2"
                        style="color: {enabled ? 'var(--color-cream-50)' : 'var(--color-primary)'}"
                        aria-hidden="true"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M12 5v7m5.657-4.657a8 8 0 11-11.314 0"
                        />
                    </svg>
                </Button>

                <!-- 2. Greeting + status -->
                <div class="flex min-w-0 flex-1 flex-col items-start gap-2 text-left">
                
                <!-- 1. Greeting -->
                <h2 class="text-2xl font-medium dark:text-white">{generateGreeting()}</h2>
                
                <!-- 2. Status Bar -->
                <div 
                    class="flex max-w-full items-start gap-2 text-sm text-gray-600 dark:text-gray-200" 
                    title={site}
                >
                    <span
                        class="mt-1.5 h-2 w-2 shrink-0 rounded-full {enabled ? 'bg-emerald-400 shadow-[0_0_6px_1px_rgba(52,211,153,0.4)]' : 'bg-slate-500'}"
                        aria-hidden="true"
                    ></span>
                    <span class="min-w-0">
                        <span class="font-medium text-gray-500 dark:text-white/80">
                            {enabled ? 'Gramflow is enabled on' : 'Gramflow is paused on'}
                        </span>
                        <span class="block font-semibold text-gray-900 [overflow-wrap:anywhere] dark:text-white">{site}</span>
                    </span>
                </div>

                </div>

            </div>
        </section>

    {/if}
</main>