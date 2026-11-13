<script lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener';
import { Button, FlowgramMark } from 'components';
import { onMount } from 'svelte';
import { Client } from '$lib/client';

const SOURCE_URL = 'https://github.com/reachneoagent-commits/gramflow';
const ISSUE_URL = 'https://github.com/reachneoagent-commits/gramflow/issues/new/choose';

let currentVersion = '';

onMount(() => {
	void loadCurrentVersion();
});

async function loadCurrentVersion() {
	try {
		currentVersion = await Client.getCurrentVersion();
	} catch (error) {
		console.error('Unable to load Flowgram Desktop version.', error);
	}
}
</script>

<section class="about">
        <div class="about-mark" aria-hidden="true"><FlowgramMark className="h-14 w-auto" /></div>
        <h1>Flow<em class="accent">gram</em></h1>
        <p class="muted">Version {currentVersion || 'unknown'}</p>
        <p>
          An open-source grammar checker that runs entirely on your device. No accounts, no
          telemetry, no cloud.
        </p>
        <div class="actions-row center">
          <Button
            unstyled
            class="button"
            type="button"
            on:click={() => void openUrl('https://github.com/reachneoagent-commits/gramflow/releases/latest')}
          >Release notes</Button>
          <Button unstyled class="button" type="button" on:click={() => void openUrl(SOURCE_URL)}>Source on GitHub</Button>
          <Button unstyled class="button" type="button" on:click={() => void openUrl(ISSUE_URL)}>Report an issue</Button>
        </div>
        <div class="about-footer">
          Flowgram is free software released under the Apache 2.0 license.
          <br />
          Copyright 2026 The Flowgram Contributors.
        </div>
      </section>
