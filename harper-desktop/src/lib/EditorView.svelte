<script lang="ts">
import type { Linter } from 'harper.js';
import { Editor } from 'harper-editor';
import { onMount } from 'svelte';
import { Client } from '$lib/client';
import { applyTheme } from '$lib/theme';
import 'harper-editor/style.css';

let linter: Linter | null = null;

onMount(() => {
	void createLinter();
	void loadTheme();

	// Follow the OS when the preference is "system".
	const media = window.matchMedia('(prefers-color-scheme: dark)');
	const onSystemChange = () => void loadTheme();
	media.addEventListener('change', onSystemChange);
	return () => media.removeEventListener('change', onSystemChange);
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

<div class="h-screen w-screen">
	{#if linter}
		<Editor {linter} />
	{/if}
</div>
