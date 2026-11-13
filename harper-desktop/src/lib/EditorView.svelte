<script lang="ts">
import type { Linter } from 'harper.js';
import { Editor } from 'harper-editor';
import { listen } from '@tauri-apps/api/event';
import { onMount } from 'svelte';
import { Client } from '$lib/client';
import { applyTheme } from '$lib/theme';
import 'harper-editor/style.css';

let linter: Linter | null = null;

type Tab = 'check' | 'humanize';
const tabs: Array<{ id: Tab; label: string }> = [
	{ id: 'check', label: 'Check' },
	{ id: 'humanize', label: 'Humanize' },
];

/** The window opens on the tab it was asked for; later requests arrive as events. */
let tab: Tab =
	(window as { __GRAMFLOW_TAB__?: string }).__GRAMFLOW_TAB__ === 'humanize' ||
	new URLSearchParams(window.location.search).get('tab') === 'humanize'
		? 'humanize'
		: 'check';

onMount(() => {
	void createLinter();
	let stopTabs: (() => void) | undefined;
	void listen<string>('editor-tab', ({ payload }) => {
		tab = payload === 'humanize' ? 'humanize' : 'check';
	}).then((stop) => (stopTabs = stop));
	void loadTheme();

	// Follow the OS when the preference is "system".
	const media = window.matchMedia('(prefers-color-scheme: dark)');
	const onSystemChange = () => void loadTheme();
	media.addEventListener('change', onSystemChange);
	return () => {
		media.removeEventListener('change', onSystemChange);
		stopTabs?.();
	};
});

async function loadTheme() {
	try {
		applyTheme(await Client.getTheme());
	} catch {
		applyTheme('system');
	}
}

async function createLinter() {
	const [{ WorkerLinter }, { slimBinaryInlined }] = await Promise.all([
		import('harper.js'),
		import('harper.js/slimBinaryInlined'),
	]);

	const nextLinter = new WorkerLinter({ binary: slimBinaryInlined });
	await nextLinter.setup();
	linter = nextLinter;
}
</script>

<div class="flex h-screen w-screen flex-col">
  <div class="editor-tabs" role="tablist" aria-label="Editor">
    {#each tabs as item}
      <button
        type="button"
        role="tab"
        class="editor-tab"
        class:selected={tab === item.id}
        aria-selected={tab === item.id}
        on:click={() => (tab = item.id)}
      >{item.label}</button>
    {/each}
  </div>
  <div class="min-h-0 flex-1 overflow-auto">
    {#if tab === 'check'}
      {#if linter}
        <Editor {linter} />
      {/if}
    {:else}
      {#await import('$lib/settings/HumanizeTab.svelte') then module}
        <svelte:component this={module.default} />
      {/await}
    {/if}
  </div>
</div>

<style>
  .editor-tabs {
    display: flex;
    gap: 4px;
    padding: 8px 12px 0;
    border-bottom: 1px solid color-mix(in srgb, currentColor 14%, transparent);
    flex: none;
  }
  :global(body[data-theme="dark"]) .editor-tabs {
    background: #15160e;
    color: #eef1dc;
  }
  :global(body[data-theme="light"]) .editor-tabs {
    background: #f5f5f0;
    color: #2b2b2b;
  }
  .editor-tab {
    padding: 8px 16px;
    border: 0;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: inherit;
    font: inherit;
    font-size: 13px;
    opacity: 0.65;
    cursor: pointer;
  }
  .editor-tab:hover {
    opacity: 1;
  }
  .editor-tab.selected {
    border-bottom-color: #b8e62e;
    opacity: 1;
    font-weight: 600;
  }
  .editor-tab:focus-visible {
    outline: 2px solid #b8e62e;
    outline-offset: -2px;
  }
</style>
