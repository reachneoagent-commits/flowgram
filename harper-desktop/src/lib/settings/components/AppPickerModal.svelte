<script lang="ts">
import { SearchIcon } from 'components';
import { onMount } from 'svelte';
import { type AppSearchResult, Client } from '$lib/client';
import AppIcon from './AppIcon.svelte';

export let bundleId = '';
export let existingBundleIds: string[];
export let isSaving = false;
export let close: () => void;
export let add: (bundleId: string) => void;

let searchResults: AppSearchResult[] = [];
let isSearching = false;
let debounceTimeout: number | null = null;
let searchRequestId = 0;

$: trimmedBundleId = bundleId.trim();
$: isDuplicate = existingBundleIds.includes(trimmedBundleId);
$: canAdd = Boolean(trimmedBundleId) && !isDuplicate && !isSaving;

onMount(() => {
	const initialSearch = window.setTimeout(() => {
		void performSearch(bundleId);
	}, 0);

	return () => window.clearTimeout(initialSearch);
});

async function performSearch(query: string) {
	const requestId = ++searchRequestId;

	isSearching = true;
	try {
		const results = await Client.searchApps(query);

		if (requestId === searchRequestId && query === bundleId) {
			searchResults = results;
		}
	} catch (error) {
		console.error('Search failed:', error);

		if (requestId === searchRequestId) {
			searchResults = [];
		}
	} finally {
		if (requestId === searchRequestId) {
			isSearching = false;
		}
	}
}

function handleInput(event: Event) {
	const target = event.target as HTMLInputElement;
	bundleId = target.value;

	if (debounceTimeout) {
		clearTimeout(debounceTimeout);
	}

	debounceTimeout = window.setTimeout(() => {
		performSearch(bundleId);
	}, 300);
}

function selectApp(selectedBundleId: string) {
	bundleId = selectedBundleId;
	void performSearch(bundleId);
}

function portal(node: HTMLElement) {
	const previous = document.activeElement as HTMLElement | null;
	document.body.appendChild(node);
	node.querySelector<HTMLInputElement>('input')?.focus();
	return {
		destroy() {
			node.remove();
			previous?.focus?.();
		},
	};
}

function trapTab(event: KeyboardEvent) {
	if (event.key !== 'Tab') return;
	const root = event.currentTarget as HTMLElement;
	const items = Array.from(
		root.querySelectorAll<HTMLElement>('button:not([disabled]), input:not([disabled]), [tabindex="0"]'),
	);
	if (items.length === 0) return;
	const first = items[0];
	const last = items[items.length - 1];
	if (event.shiftKey && document.activeElement === first) {
		event.preventDefault();
		last.focus();
	} else if (!event.shiftKey && document.activeElement === last) {
		event.preventDefault();
		first.focus();
	}
}

function submit() {
	if (canAdd) {
		add(trimmedBundleId);
	}
}
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="modal-backdrop" use:portal on:click|self={close}>
  <div
    class="modal"
    role="dialog"
    aria-modal="true"
    aria-labelledby="app-picker-title"
    tabindex="-1"
    on:keydown|stopPropagation={(event) => {
      if (event.key === "Escape") {
        close();
      }
      trapTab(event);
    }}
  >
    <div class="modal-head">
      <strong id="app-picker-title">Add application</strong>
      <span>Search for an app, select it, then click Add.</span>
    </div>
    <div class="modal-search">
      <SearchIcon className="settings-icon" />
      <input
        type="text"
        placeholder="Search for an app..."
        value={bundleId}
        disabled={isSaving}
        on:input={handleInput}
        on:keydown={(event) => {
          if (event.key === "Enter") {
            submit();
          }
        }}
      />
    </div>
    <div class="modal-list" role="listbox" aria-label="Matching applications">
      {#if isSearching}
        <div class="empty">Searching...</div>
      {:else if searchResults.length > 0}
        {#each searchResults as result}
          <div
            class="app-result"
            aria-selected={result.bundle_id === trimmedBundleId}
            role="option"
            tabindex="0"
            on:click={() => selectApp(result.bundle_id)}
            on:keydown={(event) => {
              if (event.key === "Enter" || event.key === " ") {
                selectApp(result.bundle_id);
              }
            }}
          >
            <AppIcon bundleId={result.bundle_id} name={result.name} />
            <div class="app-result-copy">
              <div class="app-result-name">{result.name}</div>
              <div class="app-result-bundle-id">{result.bundle_id}</div>
            </div>
            {#if existingBundleIds.includes(result.bundle_id)}
              <span class="pill added-pill">Already added</span>
            {/if}
          </div>
        {/each}
      {:else if trimmedBundleId}
        {#if isDuplicate}
          <div class="empty">That application is already configured.</div>
        {:else}
          <div class="empty">No matching apps found. Try typing the bundle ID directly (e.g., com.apple.TextEdit)</div>
        {/if}
      {:else}
        <div class="empty">Search for an app by name, or enter the bundle ID directly.</div>
      {/if}
    </div>
    {#if isDuplicate}
      <p class="modal-note" role="status">
        This app is already in your list. Close this window and make sure its switch is on.
      </p>
    {/if}
    <div class="modal-actions">
      <button class="button" type="button" on:click={close}>{isDuplicate ? 'Close' : 'Cancel'}</button>
      <button class="button primary" type="button" disabled={!canAdd} on:click={submit}>
        {isDuplicate ? 'Already added' : 'Add'}
      </button>
    </div>
  </div>
</div>
