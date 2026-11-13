<script lang="ts">
import {
	BookIcon,
	ChecklistIcon,
	FlowgramMark,
	GearIcon,
	GridIcon,
	InfoIcon,
	KeyIcon,
	PackageIcon,
	QuillIcon,
} from 'components';
import { FOOTER_NAV_ITEMS, MAIN_NAV_ITEMS, type SectionId } from './settings-data';

const SECTION_ICONS: Record<SectionId, typeof GearIcon> = {
	general: GearIcon,
	writing: QuillIcon,
	dictionary: BookIcon,
	shortcuts: KeyIcon,
	rules: ChecklistIcon,
	weirpacks: PackageIcon,
	integrations: GridIcon,
	humanizer: QuillIcon,
	about: InfoIcon,
};

export let active: SectionId;
</script>

<nav class="sidebar" aria-label="Settings sections">
  <div class="brand">
    <FlowgramMark className="h-5 w-auto" />
    <span>flowgram</span>
  </div>
  <div class="group">
    {#each MAIN_NAV_ITEMS as item}
      <button
        type="button"
        class:selected={active === item.id}
        aria-current={active === item.id ? 'page' : undefined}
        class="nav-item"
        on:click={() => (active = item.id)}
      >
        <span class="tile" style={`--tile-gradient: ${item.gradient}`}>
          <svelte:component this={SECTION_ICONS[item.id]} className="settings-icon" />
        </span>
        <span class="label">{item.label}</span>
      </button>
    {/each}
  </div>

  <div class="separator"></div>

  <div class="group">
    {#each FOOTER_NAV_ITEMS as item}
      <button
        type="button"
        class:selected={active === item.id}
        aria-current={active === item.id ? 'page' : undefined}
        class="nav-item"
        on:click={() => (active = item.id)}
      >
        <span class="tile" style={`--tile-gradient: ${item.gradient}`}>
          <svelte:component this={SECTION_ICONS[item.id]} className="settings-icon" />
        </span>
        <span class="label">{item.label}</span>
      </button>
    {/each}
  </div>
</nav>

<style>
  .sidebar {
    width: 208px;
    flex: 0 0 208px;
    background: var(--settings-bg-sidebar);
    border-right: 0.5px solid var(--settings-line-strong);
    display: flex;
    flex-direction: column;
    padding: 18px 8px 12px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 4px 10px 18px;
    color: var(--settings-ink);
    font-size: 20px;
    font-weight: 600;
    letter-spacing: -0.05em;
    line-height: 1;
  }

  .group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .separator {
    height: 1px;
    margin: 14px 10px 12px;
    background: var(--settings-line);
  }

  .nav-item {
    width: 100%;
    height: 36px;
    display: flex;
    align-items: center;
    gap: 10px;
    border: 0;
    border-radius: 12px;
    padding: 0 10px;
    background: transparent;
    color: var(--settings-ink);
    font: inherit;
    font-size: 14px;
    text-align: left;
    cursor: default;
  }

  .nav-item:hover {
    background: var(--settings-line);
  }

  .nav-item.selected {
    background: var(--settings-accent);
    color: var(--settings-on-accent, #2b2c23);
    font-weight: 600;
  }

  .nav-item:focus-visible {
    outline: 3px solid var(--settings-focus, #2b2c23);
    outline-offset: 2px;
  }

  .tile {
    width: 22px;
    height: 22px;
    flex: 0 0 22px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    background: transparent;
    color: inherit;
  }

  .tile :global(.settings-icon) {
    transform: scale(1.1);
  }

  .label {
    min-width: 0;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
