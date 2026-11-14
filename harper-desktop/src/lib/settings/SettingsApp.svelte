<script lang="ts">
import { onMount } from 'svelte';
import { Client } from '$lib/client';
import { applyTheme } from '$lib/theme';
import SettingsSidebar from './SettingsSidebar.svelte';
import AccessibilityBanner from './components/AccessibilityBanner.svelte';
import CoreRulesBanner from './components/CoreRulesBanner.svelte';
import './settings.css';
import AboutPage from './pages/AboutPage.svelte';
import DictionaryPage from './pages/DictionaryPage.svelte';
import GeneralPage from './pages/GeneralPage.svelte';
import GettingStartedPage from './pages/GettingStartedPage.svelte';
import HumanizerPage from './pages/HumanizerPage.svelte';
import IntegrationsPage from './pages/IntegrationsPage.svelte';
import RulesPage from './pages/RulesPage.svelte';
import ShortcutsPage from './pages/ShortcutsPage.svelte';
import WeirpacksPage from './pages/WeirpacksPage.svelte';
import WritingPage from './pages/WritingPage.svelte';
import type { SectionId } from './settings-data';

let active: SectionId = 'general';
let contentEl: HTMLElement;
let isLoadingOnboarding = true;
let onboardingCompleted = false;

const titleMap: Record<SectionId, string> = {
	general: 'General',
	writing: 'Writing',
	dictionary: 'Dictionary',
	shortcuts: 'Shortcuts',
	rules: 'Rules',
	weirpacks: 'Weirpacks',
	integrations: 'Integrations',
	humanizer: 'Humanizer',
	about: 'About',
};

onMount(() => {
	void loadOnboardingState();
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
	} catch (error) {
		console.error('Unable to load theme.', error);
		applyTheme('system');
	}
}

async function loadOnboardingState() {
	try {
		onboardingCompleted = await Client.getOnboardingCompleted();
	} catch (error) {
		console.error('Unable to load onboarding state.', error);
	} finally {
		isLoadingOnboarding = false;
	}
}

const pageHeads: Partial<Record<SectionId, { lead: string; accent: string; sub: string }>> = {
	general: { lead: 'Make it', accent: 'yours', sub: 'How Gramflow looks, behaves and starts.' },
	dictionary: {
		lead: 'Your',
		accent: 'dictionary',
		sub: 'Words Gramflow should never flag again.',
	},
	integrations: {
		lead: 'Where it',
		accent: 'works',
		sub: 'Choose the apps where Gramflow checks your writing.',
	},
	rules: {
		lead: 'What Gramflow',
		accent: 'checks',
		sub: 'Gramflow starts with our recommended settings, so you don\'t need to change anything. Turn a rule off if it gets in your way.',
	},
	shortcuts: {
		lead: 'Keyboard',
		accent: 'shortcuts',
		sub: 'Move faster without leaving the keys.',
	},
	humanizer: {
		lead: 'Rewrite in',
		accent: 'your voice',
		sub: 'Rewrite AI-sounding text on this computer. Nothing you write leaves it.',
	},
	weirpacks: { lead: 'Style', accent: 'guides', sub: 'Rule bundles that enforce a house style.' },
};

$: title = titleMap[active];
$: pageHead = pageHeads[active];

$: if (contentEl && active) {
	contentEl.scrollTop = 0;
}
</script>

{#if isLoadingOnboarding}
  <div class="settings-shell">
    <main class="content" aria-label="Settings">
      <p role="status">Loading settings...</p>
    </main>
  </div>
{:else if !onboardingCompleted}
  <GettingStartedPage onComplete={() => (onboardingCompleted = true)} />
{:else}
  <div class="settings-shell">
    <SettingsSidebar bind:active />

    <main bind:this={contentEl} class="content" aria-label={title}>
      {#if pageHead}
        <header class="page-head">
          <h1>{pageHead.lead} <em class="accent">{pageHead.accent}</em></h1>
          <p>{pageHead.sub}</p>
        </header>
      {/if}
      <AccessibilityBanner />
      <CoreRulesBanner />
      {#if active === "general"}
        <GeneralPage />
      {:else if active === "writing"}
        <WritingPage />
      {:else if active === "dictionary"}
        <DictionaryPage />
      {:else if active === "shortcuts"}
        <ShortcutsPage />
      {:else if active === "rules"}
        <RulesPage />
      {:else if active === "weirpacks"}
        <WeirpacksPage />
      {:else if active === "integrations"}
        <IntegrationsPage />
      {:else if active === "humanizer"}
        <HumanizerPage />
      {:else if active === "about"}
        <AboutPage />
      {/if}
    </main>
  </div>
{/if}
