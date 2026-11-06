<script lang="ts">
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
	'h-7 w-12 flex-none cursor-pointer rounded-full border-2 border-foreground p-0.5 transition-colors focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-ring motion-reduce:transition-none disabled:cursor-not-allowed disabled:opacity-50';
const uncheckedClasses = 'bg-background';
const checkedClasses = 'bg-primary';
const thumbClasses =
	'block size-5 rounded-full transition-transform duration-150 ease-[ease] motion-reduce:transition-none';

$: classes = [
	'toggle',
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
		role="switch"
		aria-checked={checked}
		{disabled}
		{...restProps}
		on:click={(event) => dispatch('click', event)}
	>
		<span
			class={`${thumbClasses} ${checked ? 'translate-x-5 bg-[#2b2c23]' : 'translate-x-0 bg-foreground'}`}
		></span>
	</button>
{:else}
	<label
		class="flex items-center text-sm font-medium {disabled ? 'cursor-not-allowed text-gray-400 dark:text-gray-500' : 'text-gray-900 dark:text-gray-300'} {restClass ?? ''} {className}"
	>
		<input class="peer sr-only" type="checkbox" role="switch" bind:checked {disabled} {...restProps} />
		<span
			aria-hidden="true"
			class="relative me-3 h-6 w-11 shrink-0 rounded-full bg-gray-200 after:absolute after:start-[2px] after:top-0.5 after:h-5 after:w-5 after:rounded-full after:border after:border-gray-300 after:bg-white after:transition-all after:content-[''] peer-checked:bg-primary-600 peer-checked:after:translate-x-full peer-checked:after:border-white peer-focus:ring-4 peer-focus:ring-primary-300 peer-disabled:cursor-not-allowed peer-disabled:opacity-50 peer-disabled:grayscale dark:bg-gray-700 dark:peer-focus:ring-primary-800 rtl:peer-checked:after:-translate-x-full motion-reduce:after:transition-none"
		></span>
		<slot />
	</label>
{/if}
