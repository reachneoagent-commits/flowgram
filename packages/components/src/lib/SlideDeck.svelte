<script lang="ts">
import { onMount, tick } from 'svelte';
import Button from './Button.svelte';
import GramflowMark from './GramflowMark.svelte';
import ChevronLeftIcon from './icons/ChevronLeftIcon.svelte';
import ChevronRightIcon from './icons/ChevronRightIcon.svelte';

/**
 * Slide deck shell: `slideProgress` runs from 0 (first slide) to 1 (last slide)
 * and controls navigation visibility. The default slot holds slide-specific
 * content; the `outside` slot renders below the clipped panel. Set `showAside`
 * to place the `aside` slot to the right of the text on desktop or below it on
 * mobile. `onBack` and `onNext` synchronously update the parent's active slide
 * before focus moves.
 * `nextDisabled` blocks forward navigation through both the button and keyboard
 * shortcuts without preventing users from going back. When `onFinish` is set, the last slide shows
 * a primary `finishLabel` button in the footer, right where Next appeared on earlier slides. Forward navigation plays a
 * diagonal flash across the slide content unless the user prefers reduced motion.
 * The deck switches to its dark palette under a `.dark` ancestor through
 * Tailwind `dark:` variants.
 */
export let title: string;
export let lede: string;
export let slideProgress: number;
export let onBack: () => void;
export let onNext: () => void;
export let nextDisabled = false;
export let showAside = false;
/** On the last slide the primary button becomes this action, in the same place Next occupied. */
export let onFinish: (() => void) | undefined = undefined;
export let finishLabel = 'Finish';
export let finishDisabled = false;
/** Fill the whole window with the panel (no outer margin, rounded corners or shadow), for app windows. */
export let fullBleed = false;

let heading: HTMLElement;

/** Titles read as plain text plus one serif-italic accent word (the brand's headline pattern). */
$: words = title.trim().split(/\s+/);
$: leadWords = words.length > 1 ? words.slice(0, -1).join(' ') : '';
$: accentWord = words[words.length - 1] ?? '';

/**
 * Number of flashes played. Each increment recreates the flash overlay through
 * `{#key}`, which restarts its CSS animation. Pointer presses on Next increment it
 * on press down; keyboard navigation increments it when the step changes.
 */
let flashCount = 0;

onMount(() => {
	window.addEventListener('keydown', handleKeydown);
	return () => window.removeEventListener('keydown', handleKeydown);
});

function canAdvance() {
	return slideProgress < 1 && !nextDisabled;
}

/**
 * Move within the walkthrough bounds, then focus the new heading after rendering.
 * `flash` is false for pointer clicks because their press down already played it.
 */
async function changeStep(direction: -1 | 1, flash = direction === 1) {
	if (direction === -1 ? slideProgress <= 0 : !canAdvance()) return;
	if (direction === -1) onBack();
	else {
		onNext();
		if (flash) flashCount += 1;
	}
	await tick();
	heading?.focus();
}

/** Keep navigation shortcuts out of editors and controls, including those inside shadow roots. */
function handleKeydown(event: KeyboardEvent) {
	if (
		event.defaultPrevented ||
		event.isComposing ||
		event.repeat ||
		event.altKey ||
		event.ctrlKey ||
		event.metaKey ||
		event.shiftKey ||
		event
			.composedPath()
			.some(
				(target) =>
					target instanceof Element &&
					target.closest(
						'input, textarea, select, button, a, summary, video, [contenteditable]:not([contenteditable="false"]), [role="button"], [role="textbox"]',
					),
			)
	) {
		return;
	}

	if (event.key === 'ArrowRight' || event.key === 'Enter') {
		event.preventDefault();
		void changeStep(1);
	} else if (event.key === 'ArrowLeft') {
		event.preventDefault();
		void changeStep(-1);
	}
}
</script>

<main class={fullBleed ? 'h-dvh overflow-hidden bg-background text-foreground scheme-light dark:scheme-dark' : 'grid h-dvh grid-cols-1 place-items-center overflow-y-auto bg-[#dce99b] px-6 py-6 text-foreground scheme-light [scrollbar-gutter:stable_both-edges] sm:py-10 dark:bg-[#0c0d07] dark:scheme-dark'}>
	<!-- Viewport-based width keeps the panel unchanged when reserving scrollbar gutters. -->
	<div class={fullBleed ? 'relative h-full w-full' : 'relative w-[calc(100vw-3rem)] max-w-[60rem]'}>
		<div class={fullBleed ? 'flex h-full flex-col overflow-hidden bg-background text-foreground' : 'overflow-hidden rounded-[28px] border border-border bg-background text-foreground shadow-[0_30px_60px_-30px_rgba(43,44,35,0.45)]'}>
			<div
				class="h-1.5 bg-muted"
				role="progressbar"
				aria-label="Setup progress"
				aria-valuemin={0}
				aria-valuemax={100}
				aria-valuenow={Math.round(slideProgress * 100)}
			>
				<div
					class="h-full bg-primary transition-[width] duration-200 ease-out motion-reduce:transition-none"
					style:width={`${slideProgress * 100}%`}
				></div>
			</div>

			<div class={fullBleed ? 'relative min-h-0 flex-1 overflow-y-auto px-8 pt-6 pb-6 sm:px-14' : 'relative overflow-hidden px-8 pt-7 pb-8 sm:px-14'}>
				<div class="flex items-center gap-2.5 text-foreground">
					<GramflowMark className="h-6 w-auto" />
					<span class="text-2xl leading-none font-semibold tracking-[-0.05em]">gramflow</span>
				</div>

				<div class={fullBleed ? 'grid pt-6' : 'grid min-h-96 pt-10'}>
					{#key slideProgress}
						<div class={showAside ? 'grid items-center gap-10 md:grid-cols-[minmax(0,1fr)_minmax(0,16rem)]' : ''}>
							<div class="min-w-0 self-start">
								<svelte:element
									this={slideProgress === 0 ? 'h1' : 'h2'}
									bind:this={heading}
									tabindex="-1"
									id="step-heading"
									class={fullBleed ? 'm-0! p-0! text-4xl! font-medium! leading-[1]! tracking-[-0.05em]! text-pretty text-foreground focus:outline-none sm:text-5xl!' : 'm-0! p-0! text-5xl! font-medium! leading-[0.98]! tracking-[-0.05em]! text-pretty text-foreground focus:outline-none sm:text-6xl!'}
								>
									{#if leadWords}{leadWords}&nbsp;{/if}<em class="font-serif text-[1.08em] font-normal italic tracking-[-0.03em]">{accentWord}</em>
								</svelte:element>
								<p class={fullBleed ? 'mx-0! mb-0! mt-4! max-w-3xl text-base leading-relaxed! font-normal text-pretty text-muted-foreground sm:text-lg' : 'mx-0! mb-0! mt-6! max-w-xl text-lg leading-relaxed! font-normal text-pretty text-muted-foreground sm:text-xl'}>{lede}</p>
								<slot />
							</div>
							{#if showAside}
								<div class="flex min-w-0 items-center justify-center self-stretch py-3 md:px-3">
									<slot name="aside" />
								</div>
							{/if}
						</div>
					{/key}
				</div>

				{#key flashCount}
					{#if flashCount > 0}
						<div class="slide-flash pointer-events-none absolute inset-0 z-10" aria-hidden="true"></div>
					{/if}
				{/key}
			</div>

			<div class="flex items-center justify-end gap-2.5 border-t border-border bg-muted px-8 py-4 sm:px-14">
				{#if slideProgress > 0}
					<button
						type="button"
						class="inline-grid size-11 cursor-pointer place-items-center rounded-xl border border-input bg-background text-foreground transition-colors hover:bg-muted focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-ring motion-reduce:transition-none"
						aria-label="Back"
						on:click={() => changeStep(-1)}
					>
						<ChevronLeftIcon className="size-4" />
					</button>
				{/if}
				{#if slideProgress < 1}
					<Button size="lg" color="primary" class="gap-3 rounded-xl py-1.5! pr-1.5! pl-5! text-base! font-medium! motion-reduce:transition-none" disabled={nextDisabled}
						onpointerdown={(event: PointerEvent) => {
							if (event.button === 0 && canAdvance()) flashCount += 1;
						}}
						onclick={(event: MouseEvent) => changeStep(1, event.detail === 0)}
					>
						<span>Next</span>
						<span class="inline-grid size-9 place-items-center rounded-lg bg-[#2b2c23] text-white">
							<ChevronRightIcon className="size-4" />
						</span>
					</Button>
				{:else if onFinish}
					<Button size="lg" color="primary" class="gap-3 rounded-xl py-1.5! pr-1.5! pl-5! text-base! font-medium! motion-reduce:transition-none" disabled={finishDisabled}
						onclick={() => onFinish?.()}
					>
						<span>{finishLabel}</span>
						<span class="inline-grid size-9 place-items-center rounded-lg bg-[#2b2c23] text-white">
							<ChevronRightIcon className="size-4" />
						</span>
					</Button>
				{/if}
			</div>
		</div>

		<!-- Keep supplementary content outside the clipped panel and its centering calculation. -->
		<slot name="outside" />
	</div>
</main>
