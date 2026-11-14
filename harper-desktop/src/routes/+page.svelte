<script lang="ts">
import '../app.css';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { onMount } from 'svelte';

let isSettings = false;
let isHumanize = false;
let isViewResolved = false;

function hasHumanizeRoute() {
	return new URLSearchParams(window.location.search).get('view') === 'humanize';
}

function hasSettingsRoute() {
	return (
		new URLSearchParams(window.location.search).get('view') === 'settings' ||
		window.location.hash === '#settings'
	);
}

onMount(() => {
	let currentWindowLabel = '';

	try {
		currentWindowLabel = getCurrentWindow().label;
	} catch {
		currentWindowLabel = '';
	}

	isHumanize = currentWindowLabel === 'humanize' || hasHumanizeRoute();
	isSettings = !isHumanize && (currentWindowLabel === 'settings' || hasSettingsRoute());
	isViewResolved = true;

	if (isSettings || isHumanize) {
		document.body.classList.add('settings-view');
	}

	return () => {
		document.body.classList.remove('settings-view');
	};
});
</script>

{#if isViewResolved}
  {#if isHumanize}
    {#await import("$lib/settings/HumanizeApp.svelte") then module}
      <svelte:component this={module.default} />
    {/await}
  {:else if isSettings}
    {#await import("$lib/settings/SettingsApp.svelte") then module}
      <svelte:component this={module.default} />
    {/await}
  {:else}
    {#await import("$lib/EditorView.svelte") then module}
      <svelte:component this={module.default} />
    {/await}
  {/if}
{/if}
