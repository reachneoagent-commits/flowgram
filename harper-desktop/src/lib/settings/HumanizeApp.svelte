<script lang="ts">
import { listen } from '@tauri-apps/api/event';
import { Button, Panel } from 'components';
import { onMount } from 'svelte';
import { Client, type HumanizerRewrite } from '$lib/client';
import { humanizer, refreshHumanizer } from '$lib/humanizer';
import { applyTheme } from '$lib/theme';
import './settings.css';

let draft = '';
let result = '';
let numbers: HumanizerRewrite['numbers'] = null;
let busy = false;
let error = '';
let copied = false;
/** The draft came from selected text in another app, so the result can be pasted back there. */
let fromSelection = false;
let draftEl: HTMLTextAreaElement;

$: status = $humanizer.status;
$: ready = Boolean(status && status.engine_installed && status.active);
$: wordCount = draft.trim() ? draft.trim().split(/\s+/).length : 0;

onMount(() => {
	void refreshHumanizer();
	void Client.getTheme().then(applyTheme).catch(() => applyTheme('system'));

	const stops: Array<() => void> = [];
	void listen<string>('humanizer-text', ({ payload }) => (result += payload)).then((stop) =>
		stops.push(stop),
	);
	void listen<string | null>('humanize-input', ({ payload }) => receive(payload)).then((stop) =>
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

<div class="settings-shell">
  <main class="content" aria-label="Humanize">
    <header class="page-head">
      <h1>Rewrite in <em class="accent">your voice</em></h1>
      <p>Paste a draft, or select text in any app and press the Humanize shortcut. Everything runs on this computer.</p>
    </header>

    {#if !$humanizer.loading && !ready}
      <Panel>
        <div class="app-row">
          <div class="grow">
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
            <Button size="sm" on:click={() => Client.humanizerOpenSettings()}>Open settings</Button>
          {/if}
        </div>
      </Panel>
    {/if}

    <div class="humanize-grid">
      <section class="stanza">
        <div class="eyebrow">Draft {#if wordCount}<span class="humanize-count">· {wordCount} words</span>{/if}</div>
        <textarea
          class="text-field humanize-text"
          bind:this={draftEl}
          bind:value={draft}
          placeholder="Paste the text you want to rewrite"
          aria-label="Draft"
          disabled={busy}
        ></textarea>
        <div class="inline-row">
          <Button disabled={busy || !draft.trim() || !ready} on:click={rewrite}>
            {busy ? 'Rewriting...' : 'Rewrite'}
          </Button>
          {#if busy}
            <Button color="white" on:click={() => Client.humanizerCancelRewrite()}>Stop</Button>
          {/if}
          <span class="humanize-hint">Cmd or Ctrl + Enter</span>
        </div>
      </section>

      <section class="stanza">
        <div class="eyebrow">Rewrite</div>
        <textarea
          class="text-field humanize-text"
          bind:value={result}
          placeholder={busy ? 'Working on it...' : 'The rewrite shows up here'}
          aria-label="Rewrite"
          readonly={busy}
        ></textarea>
        <div class="inline-row">
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
        <p class="result-summary" role="alert">
          <span class="pill amber">Check the numbers</span>
          {#if numbers.missing.length}The rewrite dropped {numbers.missing.join(', ')}.{/if}
          {#if numbers.added.length}It added {numbers.added.join(', ')}.{/if}
        </p>
      {:else if /\d/.test(draft)}
        <p class="result-summary">Numbers match the draft.</p>
      {/if}
    {/if}
    {#if error}
      <p class="result-summary" role="alert">{error}</p>
    {/if}
  </main>
</div>

<style>
  .humanize-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 24px;
    margin-top: 16px;
  }
  .humanize-text {
    width: 100%;
    min-height: 280px;
    resize: vertical;
    line-height: 1.6;
  }
  .humanize-count,
  .humanize-hint {
    color: var(--settings-ink-3);
    font-size: 12px;
  }
  @media (max-width: 760px) {
    .humanize-grid {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
