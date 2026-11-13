<script lang="ts">
import { Button, CloseIcon, IconButton, SearchField, SearchIcon, Toggle } from 'components';
import type { LintConfig, StructuredLintConfig, StructuredLintSetting } from 'harper.js';
import { onMount } from 'svelte';
import { Client, LINT_CONFIG_CHANGED } from '$lib/client';
import ConfirmDialog from '../components/ConfirmDialog.svelte';

type RuleItem = { id: string; name: string; desc: string };
type RuleGroup = { id: string; title: string; desc: string; rules: RuleItem[] };
type MatchedRuleGroup = RuleGroup & { matchedRules: RuleItem[] };
type PendingOff = { title: string; message: string; ruleIds: string[] };

/** Rules most people rely on. Turning one off asks first. */
const CORE_RULES: Record<string, string> = {
	SpellCheck: 'Spell checking',
};

let lintConfig: LintConfig | null = null;
/** Gramflow's recommended setting for every rule. */
let recommended: LintConfig | null = null;
let descriptions: Record<string, string> = {};
let structuredLintConfig: StructuredLintConfig | null = null;
let rulesSearch = '';
let expandedGroups: Record<string, boolean> = {};
let isLintConfigLoading = true;
let isLintConfigSaving = false;
let lintConfigError = '';
let pendingOff: PendingOff | null = null;

$: rulesQuery = rulesSearch.trim().toLowerCase();
$: ruleGroups = structuredLintConfig ? ruleGroupsFromStructuredConfig(structuredLintConfig) : [];
$: displayedRules = ruleGroups.flatMap((group) => group.rules);
$: enabledRuleCount = displayedRules.filter((rule) => isOn(rule.id, lintConfig)).length;
$: changedRuleCount = lintConfig && recommended ? changedRuleIds().length : 0;
$: filteredRuleGroups = getFilteredRuleGroups(ruleGroups, rulesQuery, descriptions);

onMount(() => {
	void loadLintConfig();

	const refreshLintConfig = () => {
		if (!isLintConfigSaving) {
			void loadLintConfig();
		}
	};

	window.addEventListener('focus', refreshLintConfig);
	// Changes made elsewhere in Settings, such as the banner's "Turn on" button.
	window.addEventListener(LINT_CONFIG_CHANGED, refreshLintConfig);

	return () => {
		window.removeEventListener('focus', refreshLintConfig);
		window.removeEventListener(LINT_CONFIG_CHANGED, refreshLintConfig);
	};
});

async function loadLintConfig() {
	isLintConfigLoading = true;
	lintConfigError = '';

	try {
		const [fetchedLintConfig, fetchedRecommended, fetchedStructured, fetchedDescriptions] =
			await Promise.all([
				Client.getLintConfig(),
				Client.getDefaultLintConfig(),
				Client.getStructuredLintConfig(),
				Client.getLintDescriptions(),
			]);

		lintConfig = fetchedLintConfig;
		recommended = fetchedRecommended;
		structuredLintConfig = fetchedStructured;
		descriptions = fetchedDescriptions;
	} catch (error) {
		lintConfigError = `Unable to load rules: ${error}`;
	} finally {
		isLintConfigLoading = false;
	}
}

function ruleLabelFromKey(key: string) {
	return key
		.replace(/([a-z0-9])([A-Z])/g, '$1 $2')
		.replace(/([A-Z]+)([A-Z][a-z])/g, '$1 $2')
		.trim();
}

function ruleGroupsFromStructuredConfig(config: StructuredLintConfig): RuleGroup[] {
	const groups: RuleGroup[] = [];
	const looseRules: RuleItem[] = [];

	for (const setting of config.settings) {
		if ('Group' in setting) {
			groups.push(...groupsFromSetting(setting.Group));
		} else {
			looseRules.push(...rulesFromSetting(setting));
		}
	}

	if (looseRules.length > 0) {
		groups.push({
			id: 'ungrouped-rules',
			title: 'Ungrouped Rules',
			desc: "Rules from the app's current lint configuration that are not assigned to a category.",
			rules: looseRules,
		});
	}

	return groups;
}

function groupsFromSetting(
	group: Extract<StructuredLintSetting, { Group: unknown }>['Group'],
): RuleGroup[] {
	const groups: RuleGroup[] = [];
	const rules: RuleItem[] = [];

	for (const setting of group.child.settings) {
		if ('Group' in setting) {
			groups.push(...groupsFromSetting(setting.Group));
		} else {
			rules.push(...rulesFromSetting(setting));
		}
	}

	if (rules.length > 0) {
		groups.unshift({
			id: `${groups.length}-${group.label}`,
			title: group.label,
			desc: group.description,
			rules,
		});
	}

	return groups;
}

function rulesFromSetting(setting: StructuredLintSetting): RuleItem[] {
	if ('Bool' in setting) {
		return [
			{
				id: setting.Bool.name,
				name: setting.Bool.label ?? ruleLabelFromKey(setting.Bool.name),
				desc: '',
			},
		];
	}

	if ('OneOfMany' in setting) {
		return setting.OneOfMany.names.map((name, index) => ({
			id: name,
			name: setting.OneOfMany.labels?.[index] ?? ruleLabelFromKey(name),
			desc: '',
		}));
	}

	return [];
}

/** First sentence of a rule's description as plain text, so rows stay short. */
function shortDescription(rule: RuleItem): string {
	const raw = descriptions[rule.id] ?? '';
	const plain = raw
		.replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
		.replace(/[`*_]/g, '')
		.replace(/\s+/g, ' ')
		.trim();
	// Some grouped rules only carry this placeholder text, which tells the reader nothing.
	if (plain.startsWith('A collection of linters')) return '';
	const firstSentence = plain.match(/^.*?[.!?](?=\s|$)/)?.[0] ?? plain;

	return firstSentence.length > 160 ? `${firstSentence.slice(0, 157)}...` : firstSentence;
}

/** `config` is a parameter (not read from the component) so Svelte re-renders when it changes. */
function isOn(ruleId: string, config: LintConfig | null = lintConfig): boolean {
	return config?.[ruleId] ?? recommended?.[ruleId] ?? false;
}

function recommendedOn(ruleId: string): boolean {
	return recommended?.[ruleId] ?? false;
}

function changedRuleIds(): string[] {
	if (!lintConfig || !recommended) return [];

	return Object.keys(recommended).filter(
		(ruleId) => (lintConfig?.[ruleId] ?? recommended?.[ruleId]) !== recommended?.[ruleId],
	);
}

function groupOnCount(group: RuleGroup, config: LintConfig | null): number {
	return group.rules.filter((rule) => isOn(rule.id, config)).length;
}

function getFilteredRuleGroups(
	groups: RuleGroup[],
	query: string,
	rulesDescriptions: Record<string, string>,
): MatchedRuleGroup[] {
	if (!query) {
		return groups.map((group) => ({ ...group, matchedRules: group.rules }));
	}

	return groups
		.map((group) => {
			const groupMatches =
				group.title.toLowerCase().includes(query) || group.desc.toLowerCase().includes(query);
			const squashed = query.replace(/\s+/g, '');
			const matchedRules = group.rules.filter(
				(rule) =>
					rule.name.toLowerCase().includes(query) ||
					rule.id.toLowerCase().includes(squashed) ||
					(rulesDescriptions[rule.id] ?? '').toLowerCase().includes(query),
			);

			if (groupMatches) {
				return { ...group, matchedRules: group.rules };
			}

			if (matchedRules.length > 0) {
				return { ...group, matchedRules };
			}

			return null;
		})
		.filter((group): group is MatchedRuleGroup => group !== null);
}

async function saveLintConfig(nextLintConfig: LintConfig) {
	const previousLintConfig = lintConfig;

	lintConfig = nextLintConfig;
	isLintConfigSaving = true;
	lintConfigError = '';

	try {
		await Client.setLintConfig(nextLintConfig);
	} catch (error) {
		lintConfig = previousLintConfig;
		lintConfigError = `Unable to save rules: ${error}`;
	} finally {
		isLintConfigSaving = false;
	}
}

/** Applies a set of rule changes, asking first when something important would be switched off. */
async function setRules(ruleIds: string[], on: boolean, groupTitle?: string) {
	if (!lintConfig) return;

	if (!on) {
		const coreRules = ruleIds.filter((ruleId) => ruleId in CORE_RULES && isOn(ruleId));
		const turningOff = ruleIds.filter((ruleId) => isOn(ruleId));

		if (coreRules.length > 0) {
			pendingOff = {
				title: `Turn off ${coreRules.map((ruleId) => CORE_RULES[ruleId]).join(' and ')}?`,
				message:
					'Gramflow will stop catching these mistakes in every app until you turn it back on.',
				ruleIds,
			};
			return;
		}

		if (groupTitle && turningOff.length > 1) {
			pendingOff = {
				title: `Turn off all ${turningOff.length} “${groupTitle}” rules?`,
				message: 'Gramflow will stop checking for these. You can turn them back on any time.',
				ruleIds,
			};
			return;
		}
	}

	await applyRules(ruleIds, on);
}

async function applyRules(ruleIds: string[], on: boolean) {
	const nextLintConfig = { ...(lintConfig ?? {}) };

	for (const ruleId of ruleIds) {
		nextLintConfig[ruleId] = on;
	}

	await saveLintConfig(nextLintConfig);
}

async function confirmPendingOff() {
	const pending = pendingOff;
	pendingOff = null;

	if (pending) await applyRules(pending.ruleIds, false);
}

/** Back to Gramflow's recommended settings for every rule, including ones not listed here. */
async function resetToRecommended() {
	if (!recommended) return;

	await saveLintConfig({ ...recommended });
}

function toggleGroup(groupId: string) {
	if (rulesQuery) return;
	expandedGroups = { ...expandedGroups, [groupId]: !expandedGroups[groupId] };
}
</script>

<section>
        <div class="rules-heading">
          <p class="rules-summary">
            <strong>{enabledRuleCount}</strong> of {displayedRules.length} rules are on.
            {#if changedRuleCount > 0}
              <strong>{changedRuleCount}</strong> {changedRuleCount === 1 ? 'differs' : 'differ'} from our recommendation.
            {:else}
              All match our recommendation.
            {/if}
          </p>
        </div>

        {#if isLintConfigLoading && !lintConfig}
          <p class="result-summary">Loading rules...</p>
        {:else if lintConfigError}
          <p class="result-summary" role="alert">{lintConfigError}</p>
        {/if}

        <div class="sticky-tools">
          <SearchField class="rule-search" placeholder="Search rules..." bind:value={rulesSearch}>
            <SearchIcon slot="leading" className="control-icon" />
            {#if rulesSearch}
              <IconButton slot="trailing" aria-label="Clear search" on:click={() => (rulesSearch = "")}>
                <CloseIcon className="control-icon" />
              </IconButton>
            {/if}
          </SearchField>
          <Button
            unstyled
            class={`button reset-recommended ${changedRuleCount > 0 ? 'primary' : ''}`}
            type="button"
            disabled={isLintConfigLoading || isLintConfigSaving || changedRuleCount === 0}
            on:click={resetToRecommended}
          >
            {changedRuleCount > 0 ? `Reset ${changedRuleCount} to recommended` : 'Reset to recommended'}
          </Button>
        </div>

        {#if rulesQuery}
          <p class="result-summary">
            {filteredRuleGroups.reduce((total, group) => total + group.matchedRules.length, 0)}
            rules match "{rulesSearch}" in {filteredRuleGroups.length} {filteredRuleGroups.length === 1 ? 'group' : 'groups'}.
          </p>
        {/if}

        <div class="rule-groups">
          {#each filteredRuleGroups as group}
            {@const expanded = rulesQuery || expandedGroups[group.id]}
            {@const onCount = groupOnCount(group, lintConfig)}
            <article class="rule-group">
              <Button unstyled class="group-head" type="button" aria-expanded={!!expanded} on:click={() => toggleGroup(group.id)}>
                <svg class:expanded class="chevron" viewBox="0 0 16 16" aria-hidden="true">
                  <path
                    fill-rule="evenodd"
                    d="M6.22 4.22a.75.75 0 0 1 1.06 0l3.25 3.25a.75.75 0 0 1 0 1.06l-3.25 3.25a.75.75 0 0 1-1.06-1.06L8.94 8 6.22 5.28a.75.75 0 0 1 0-1.06Z"
                    clip-rule="evenodd"
                  />
                </svg>
                <span class="grow">
                  <strong>{group.title}</strong>
                  <p>{group.desc}</p>
                </span>
                <span class="group-status" class:partial={onCount > 0 && onCount < group.rules.length} class:none={onCount === 0}>
                  {#if onCount === group.rules.length}
                    All {group.rules.length} on
                  {:else if onCount === 0}
                    All off
                  {:else}
                    {onCount} of {group.rules.length} on
                  {/if}
                </span>
              </Button>

              {#if expanded}
                <div class="rules-toolbar">
                  <Button unstyled class="link-button" type="button" disabled={isLintConfigSaving || onCount === group.rules.length} on:click={() => setRules(group.rules.map((rule) => rule.id), true)}>Turn all on</Button>
                  <Button unstyled class="link-button" type="button" disabled={isLintConfigSaving || onCount === 0} on:click={() => setRules(group.rules.map((rule) => rule.id), false, group.title)}>Turn all off</Button>
                </div>
                <div class="rules-list scrollable" tabindex="-1">
                  {#each group.matchedRules as rule (rule.id)}
                    {@const on = isOn(rule.id, lintConfig)}
                    <div class="rule-row">
                      <div class="grow">
                        <strong>{rule.name}</strong>
                        {#if on !== recommendedOn(rule.id)}
                          <span class="pill amber">Changed</span>
                          <Button unstyled class="link-button inline" type="button" on:click={() => setRules([rule.id], recommendedOn(rule.id))}>
                            Restore to {recommendedOn(rule.id) ? 'on' : 'off'}
                          </Button>
                        {:else if !recommendedOn(rule.id)}
                          <span class="pill">Optional</span>
                        {/if}
                        {#if shortDescription(rule)}
                          <p>{shortDescription(rule)}</p>
                        {/if}
                      </div>
                      <Toggle
                        appearance="settings"
                        checked={on}
                        disabled={isLintConfigLoading || isLintConfigSaving}
                        aria-label={`${rule.name}: ${on ? 'on' : 'off'}`}
                        on:click={() => setRules([rule.id], !on)}
                      />
                    </div>
                  {/each}
                </div>
              {/if}
            </article>
          {/each}
        </div>
      </section>

{#if pendingOff}
  <ConfirmDialog
    title={pendingOff.title}
    message={pendingOff.message}
    onCancel={() => (pendingOff = null)}
    onConfirm={confirmPendingOff}
  />
{/if}
