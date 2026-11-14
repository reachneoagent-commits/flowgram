<script lang="ts">
import { listen } from '@tauri-apps/api/event';
import { Button } from 'components';
import { onMount } from 'svelte';
import { Client, type HumanizerRewrite } from '$lib/client';
import { humanizer, refreshHumanizer } from '$lib/humanizer';

let draft = '';
let result = '';
let numbers: HumanizerRewrite['numbers'] = null;
let busy = false;
let error = '';
let copied = false;
/** The draft came from selected text in another app, so the result can be pasted back there. */
let fromSelection = false;
let draftEl: HTMLTextAreaElement;

/** Called when the person needs to set up a model first. */
export let onSetup: (() => void) | null = null;
/** Fill the window: taller text areas for the editor tab. */
export let tall = false;

$: status = $humanizer.status;
$: ready = Boolean(status && status.engine_installed && status.active);
$: wordCount = draft.trim() ? draft.trim().split(/\s+/).length : 0;

onMount(() => {
	void refreshHumanizer();

	const stops: Array<() => void> = [];
	void listen<string>('humanizer-text', ({ payload }) => (result += payload)).then((stop) =>
		stops.push(stop),
	);
	void listen<string | null>('humanize-input', ({ payload }) => {
		// The text also waits in the backend for a window that wasn't loaded yet; clear it.
		void Client.humanizerTakePending();
		receive(payload);
	}).then((stop) =>
		stops.push(stop),
	);
	void Client.humanizerTakePending().then((text) => text && receive(text));
	draftEl?.focus();

	return () => {
		for (const stop of stops) stop();
	};
});

/** Selected text arrives from the shortcut; rewrite it right away. */
function receive(text: string | null) {
	if (!text) {
		fromSelection = false;
		draftEl?.focus();
		return;
	}
	draft = text;
	fromSelection = true;
	void rewrite();
}

async function rewrite() {
	if (busy || !draft.trim()) return;
	busy = true;
	error = '';
	result = '';
	numbers = null;
	copied = false;
	try {
		const done = await Client.humanizerRewrite(draft);
		result = done.text;
		numbers = done.numbers;
	} catch (failure) {
		error = `${failure}`;
	} finally {
		busy = false;
		void refreshHumanizer();
	}
}

async function pasteDraft() {
	try {
		draft = await navigator.clipboard.readText();
		fromSelection = false;
		draftEl?.focus();
	} catch (failure) {
		error = `Couldn't read the clipboard: ${failure}`;
	}
}

function clearAll() {
	draft = '';
	result = '';
	numbers = null;
	error = '';
	fromSelection = false;
	draftEl?.focus();
}

async function copyResult() {
	try {
		await navigator.clipboard.writeText(result);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	} catch (failure) {
		error = `Couldn't copy: ${failure}`;
	}
}

async function replaceSelection() {
	error = '';
	try {
		await Client.humanizerPasteBack(result);
	} catch (failure) {
		error = `${failure}`;
	}
}

function onKeydown(event: KeyboardEvent) {
	if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
		event.preventDefault();
		void rewrite();
	}
}
</script>

<svelte:window on:keydown={onKeydown} />

<div class="hz" class:tall>
  {#if !$humanizer.loading && !ready}
    <div class="hz-setup" role="status">
      <div>
        <strong>Humanizer isn't set up yet</strong>
        <p>
          {#if status && !status.engine_installed}
            The rewriting engine isn't included in this build.
          {:else if status?.verdict.level === 'Unavailable'}
            {status.verdict.detail}
          {:else}
            Download a model first. It takes a few minutes and only happens once.
          {/if}
        </p>
      </div>
      {#if status?.verdict.level !== 'Unavailable'}
        <Button size="sm" on:click={() => (onSetup ? onSetup() : Client.humanizerOpenSettings())}>Open settings</Button>
      {/if}
    </div>
  {/if}

  <div class="hz-grid">
    <section class="hz-pane">
      <div class="hz-label">Draft {#if wordCount}<span class="hz-muted">· {wordCount} words</span>{/if}</div>
      <textarea
        class="hz-text"
        bind:this={draftEl}
        bind:value={draft}
        placeholder="Paste the text you want to rewrite"
        aria-label="Draft"
        disabled={busy}
      ></textarea>
      <div class="hz-row">
        <Button title="Cmd or Ctrl + Enter" disabled={busy || !draft.trim() || !ready} on:click={rewrite}>
          {busy ? 'Rewriting...' : 'Rewrite'}
        </Button>
        {#if busy}
          <Button color="white" on:click={() => Client.humanizerCancelRewrite()}>Stop</Button>
        {/if}
        <Button color="white" disabled={busy} on:click={pasteDraft}>Paste</Button>
        <Button color="white" disabled={busy || (!draft && !result)} on:click={clearAll}>Clear</Button>
      </div>
    </section>

    <section class="hz-pane">
      <div class="hz-label">Rewrite</div>
      <textarea
        class="hz-text"
        bind:value={result}
        placeholder={busy ? 'Working on it...' : 'The rewrite shows up here'}
        aria-label="Rewrite"
        readonly={busy}
      ></textarea>
      <div class="hz-row">
        <Button color="white" disabled={busy || !result} on:click={copyResult}>
          {copied ? 'Copied' : 'Copy'}
        </Button>
        {#if fromSelection}
          <Button disabled={busy || !result} on:click={replaceSelection}>Replace selection</Button>
        {/if}
        <Button color="white" disabled={busy || !draft.trim() || !ready} on:click={rewrite}>Regenerate</Button>
      </div>
    </section>
  </div>

  {#if numbers && !busy}
    {#if numbers.missing.length || numbers.added.length}
      <p class="hz-note" role="alert">
        <span class="hz-pill">Check the numbers</span>
        {#if numbers.missing.length}The rewrite dropped {numbers.missing.join(', ')}.{/if}
        {#if numbers.added.length}It added {numbers.added.join(', ')}.{/if}
      </p>
    {:else if /\d/.test(draft)}
      <p class="hz-note">Numbers match the draft.</p>
    {/if}
  {/if}
  {#if error}
    <p class="hz-note" role="alert">{error}</p>
  {/if}
</div>

<style>
  /* Self-contained: the editor window also shows the grammar checker, so nothing here may leak
     out, and nothing here depends on the Settings stylesheet. */
  .hz {
    --hz-ink: #2b2b2b;
    --hz-ink-2: #6b6b63;
    --hz-field: #ffffff;
    --hz-line: rgba(43, 43, 43, 0.18);
    --hz-panel: rgba(43, 43, 43, 0.05);
    color: var(--hz-ink);
    font-size: 14px;
  }
  :global(.dark) .hz {
    --hz-ink: #eef1dc;
    --hz-ink-2: #a9ad96;
    --hz-field: #2b2c22;
    --hz-line: rgba(255, 255, 255, 0.14);
    --hz-panel: rgba(255, 255, 255, 0.06);
  }
  .hz-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 24px;
    margin-top: 16px;
  }
  .hz-pane {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
  }
  .hz-label {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--hz-ink-2);
  }
  .hz-muted {
    font-weight: 400;
    letter-spacing: 0;
    text-transform: none;
  }
  .hz-text {
    width: 100%;
    min-height: 280px;
    padding: 12px 14px;
    border: 1px solid var(--hz-line);
    border-radius: 10px;
    background: var(--hz-field);
    color: var(--hz-ink);
    font: inherit;
    line-height: 1.6;
    resize: vertical;
  }
  .hz-text:focus-visible {
    outline: 2px solid #b8e62e;
    outline-offset: 1px;
  }
  .tall .hz-text {
    min-height: max(280px, calc(100vh - 330px));
  }
  .hz-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
  }
  .hz-note {
    margin: 14px 0 0;
    color: var(--hz-ink-2);
    font-size: 13px;
  }
  .hz-pill {
    display: inline-block;
    margin-right: 8px;
    padding: 2px 8px;
    border-radius: 4px;
    background: rgba(184, 230, 46, 0.25);
    color: var(--hz-ink);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .hz-setup {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 14px 16px;
    border: 1px solid var(--hz-line);
    border-radius: 10px;
    background: var(--hz-panel);
  }
  .hz-setup p {
    margin: 2px 0 0;
    color: var(--hz-ink-2);
    font-size: 13px;
  }
  @media (max-width: 760px) {
    .hz-grid {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
