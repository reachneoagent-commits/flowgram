<script lang="ts">
/** The Gramflow logo mark. Fills with the current text colour so it works on any surface. */
export let className = 'h-6 w-auto';
export let label = '';
</script>

<svg
	class={className}
	viewBox="0 0 53.97 54.65"
	fill="currentColor"
	role={label ? 'img' : undefined}
	aria-label={label || undefined}
	aria-hidden={label ? undefined : 'true'}
>
	<rect x="35.67" width="18.31" height="18.31" rx="1.58" ry="1.58" />
	<path
		d="M52.39,28.87h-15.14c-.87,0-1.58.71-1.58,1.58v15.14c0,.87.71,1.58,1.58,1.58h5.26c1.11,0,1.87,1.11,1.48,2.14l-1.2,3.17c-.39,1.03.37,2.14,1.48,2.14h5.73c.65,0,1.24-.4,1.47-1.01h0c1.66-4.26,2.51-8.8,2.51-13.37v-9.81c0-.87-.71-1.58-1.58-1.58Z"
	/>
	<path
		d="M19.49,27.39c0-9.63,2.38-18.47,6.34-25.33.52-.89-.18-2.02-1.21-1.91C10.79,1.52,0,13.19,0,27.39s10.8,25.88,24.63,27.26c1.03.1,1.72-1.02,1.2-1.92-3.97-6.86-6.35-15.7-6.35-25.34Z"
	/>
</svg>
