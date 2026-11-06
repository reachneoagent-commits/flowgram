<script lang="ts">
import { Checkbox as FlowbiteCheckbox } from 'flowbite-svelte';
import { createEventDispatcher } from 'svelte';
import type { HTMLButtonAttributes } from 'svelte/elements';

export let checked = false;
export let disabled: HTMLButtonAttributes['disabled'] = undefined;
export let appearance: 'default' | 'settings' = 'default';
export let className = '';

let restClass: string | undefined;
let restProps: Record<string, unknown> = {};
const dispatch = createEventDispatcher<{ click: MouseEvent }>();

$: ({ class: restClass, ...restProps } = $$restProps);
const baseClasses =
	'size-5 flex-none grid place-items-center rounded-md border-2 p-0 text-[#2b2c23] [&>svg]:size-[13px] [&>svg]:flex-none';
const uncheckedClasses = 'border-[var(--settings-ink-3,#6b6e60)] bg-[var(--settings-surface,#fff)]';
const checkedClasses = 'border-[#2b2c23] bg-[var(--settings-accent,#b8e62e)]';

$: classes = [
	'checkbox',
	baseClasses,
	checked && 'checked',
	checked ? checkedClasses : uncheckedClasses,
	restClass,
	className,
]
	.filter(Boolean)
	.join(' ');
</script>

{#if appearance === 'settings'}
	<button
		class={classes}
		type="button"
		role="checkbox"
		aria-checked={checked}
		{disabled}
		{...restProps}
		on:click={(event) => dispatch('click', event)}
	>
		<slot />
	</button>
{:else}
	<FlowbiteCheckbox bind:checked {disabled} class={restClass} {...restProps}>
		<slot />
	</FlowbiteCheckbox>
{/if}
