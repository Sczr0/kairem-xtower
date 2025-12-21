<svelte:options runes={false} />

<script lang="ts">
	import { onMount } from 'svelte';
	import { colorBlindEnabled, initColorBlind, toggleColorBlind } from '$lib/a11y';

	onMount(() => {
		initColorBlind();
	});
</script>

<button
	type="button"
	class="a11y-toggle btn"
	on:click={toggleColorBlind}
	aria-pressed={$colorBlindEnabled}
	aria-label={$colorBlindEnabled ? '关闭色盲模式' : '开启色盲模式'}
	title={$colorBlindEnabled ? '关闭色盲模式' : '开启色盲模式'}
>
	<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
		<path
			d="M2 12s3.5-7 10-7s10 7 10 7s-3.5 7-10 7S2 12 2 12Z"
			stroke-linecap="round"
			stroke-linejoin="round"
		/>
		<path d="M12 15a3 3 0 1 0 0-6a3 3 0 0 0 0 6Z" stroke-linecap="round" stroke-linejoin="round" />
	</svg>
	<span class="label">色盲</span>
</button>

<style>
	.a11y-toggle {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-radius: 999px;
	}

	.icon {
		width: 16px;
		height: 16px;
	}

	.label {
		font-size: 0.86rem;
		color: var(--muted);
	}

	@media (max-width: 520px) {
		.label {
			display: none;
		}
	}
</style>

