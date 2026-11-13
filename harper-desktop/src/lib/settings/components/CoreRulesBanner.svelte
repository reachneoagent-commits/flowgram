<script lang="ts">
import { Button } from 'components';
import { onMount } from 'svelte';
import { Client, LINT_CONFIG_CHANGED } from '$lib/client';

/** Rules most people rely on. If one is off, Gramflow looks broken, so say so. */
const CORE_RULES: Record<string, string> = {
	SpellCheck: 'Spell checking',
};

let disabled: string[] = [];
let isFixing = false;

onMount(() => {
	void check();
	window.addEventListener('focus', check);
	window.addEventListener(LINT_CONFIG_CHANGED, check);
	return () => {
		window.removeEventListener('focus', check);
		window.removeEventListener(LINT_CONFIG_CHANGED, check);
	};
});

async function check() {
	try {
		const config = await Client.getLintConfig();
		disabled = Object.keys(CORE_RULES).filter((rule) => config[rule] === false);
	} catch {
		disabled = [];
	}
}

async function turnOn() {
	isFixing = true;
	try {
		const config = await Client.getLintConfig();
		for (const rule of disabled) config[rule] = true;
		await Client.setLintConfig(config);
		await check();
	} finally {
		isFixing = false;
	}
}
</script>

{#if disabled.length > 0}
  <div class="core-rules-banner" role="alert">
    <p>
      <strong>{disabled.map((rule) => CORE_RULES[rule]).join(', ')} {disabled.length > 1 ? 'are' : 'is'} turned off.</strong>
      Gramflow won't catch these mistakes until you switch it back on.
    </p>
    <Button disabled={isFixing} on:click={turnOn}>{isFixing ? 'Turning on...' : 'Turn on'}</Button>
  </div>
{/if}

<style>
  .core-rules-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 16px;
    padding: 12px 14px;
    border: 1px solid #e06c4c;
    border-radius: 10px;
    background: rgba(224, 108, 76, 0.1);
    font-size: 13px;
  }
  .core-rules-banner p {
    margin: 0;
  }
</style>
