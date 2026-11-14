<script lang="ts">
import { isTauri } from '@tauri-apps/api/core';
import { platform } from '@tauri-apps/plugin-os';
import { Button } from 'components';
import { onMount } from 'svelte';
import { Client } from '$lib/client';

/** Clears a stale macOS Accessibility entry (for example after an update changed the app's signature). */
const RESET_COMMAND = 'tccutil reset Accessibility com.gramflow.app';

let missing = false;
let isWorking = false;
let copied = false;
let message = '';

onMount(() => {
	const isMac = isTauri() ? platform() === 'macos' : navigator.userAgent.includes('Macintosh');
	if (!isMac) return;

	void check();
	// Notice as soon as the user switches Gramflow on in System Settings.
	const timer = setInterval(check, 2000);
	window.addEventListener('focus', check);

	return () => {
		clearInterval(timer);
		window.removeEventListener('focus', check);
	};
});

async function check() {
	try {
		const status = await Client.getAccessibilityPermissionStatus();
		const wasMissing = missing;
		missing = status === 'NotGranted';

		// Access arrived while Gramflow was running: start checking text right away.
		if (wasMissing && !missing) await Client.startHighlighterService();
	} catch {
		// Leave the banner as it is; the next check will try again.
	}
}

async function openSystemSettings() {
	isWorking = true;
	message = '';
	try {
		await Client.requestAccessibilityPermission();
		await check();
	} catch (error) {
		message = `${error}`;
	} finally {
		isWorking = false;
	}
}

async function copyReset() {
	try {
		await navigator.clipboard.writeText(RESET_COMMAND);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	} catch {
		// Clipboard unavailable; the command is shown on screen to copy by hand.
	}
}
</script>

{#if missing}
  <div class="accessibility-banner" role="alert">
    <div>
      <p>
        <strong>Gramflow can't see what you write yet.</strong>
        Switch Gramflow on in System Settings → Privacy &amp; Security → Accessibility. Gramflow notices and starts by itself.
      </p>
      <details>
        <summary>Already switched on but it still doesn't work?</summary>
        <p>
          macOS keeps an old entry after Gramflow is updated or reinstalled. Quit Gramflow, run this in
          Terminal, then open Gramflow and allow access again:
        </p>
        <p class="command"><code>{RESET_COMMAND}</code>
          <Button color="white" size="sm" on:click={copyReset}>{copied ? 'Copied' : 'Copy'}</Button>
        </p>
      </details>
      {#if message}<p role="alert">{message}</p>{/if}
    </div>
    <Button disabled={isWorking} on:click={openSystemSettings}>Open System Settings</Button>
  </div>
{/if}

<style>
  .accessibility-banner {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 16px;
    padding: 12px 14px;
    border: 1px solid #e06c4c;
    border-radius: 10px;
    background: rgba(224, 108, 76, 0.1);
    font-size: 13px;
  }
  p {
    margin: 0 0 6px;
  }
  details {
    margin-top: 6px;
  }
  summary {
    cursor: pointer;
  }
  .command {
    display: flex;
    align-items: center;
    gap: 8px;
  }
</style>
