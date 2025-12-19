<svelte:options runes={false} />

<script lang="ts">
	import { colorToCss, Color, type ColorId } from '$lib/colors';

	const indices = Array.from({ length: 25 }, (_, i) => i);

	export let grid: ColorId[] = Array.from({ length: 25 }, () => Color.White);
	export let checkedMask = 0;
	export let cellOk: boolean[] = Array.from({ length: 25 }, () => true);
	export let onToggle: (index: number) => void = () => {};
	export let onHover: (index: number | null) => void = () => {};

	// 注意：不要把 checkedMask 依赖藏在函数闭包里，否则 Svelte 可能不会在 mask 更新时重算模板。
	// 这里用响应式语句显式建立依赖关系，保证勾选样式能即时更新。
	$: checkedFlags = indices.map((i) => ((checkedMask >>> 0) & (1 << i)) !== 0);
	$: blackFlags = indices.map((i) => grid[i] === Color.Black);

	function handleMouseEnter(i: number) {
		onHover(i);
	}

	function handleMouseLeave() {
		onHover(null);
	}
</script>

<div class="matrix" role="grid" aria-label="5x5 矩阵">
	{#each indices as i}
		<div class="cell-wrapper">
			<button
				type="button"
				class="cell {checkedFlags[i] ? 'checked' : ''} {cellOk[i] ? '' : 'invalid'}"
				style="--cell-color: {colorToCss(grid[i])}"
				aria-pressed={checkedFlags[i]}
				disabled={blackFlags[i]}
				on:click={() => onToggle(i)}
				on:mouseenter={() => handleMouseEnter(i)}
				on:mouseleave={handleMouseLeave}
				on:focus={() => handleMouseEnter(i)}
				on:blur={handleMouseLeave}
			>
				<div class="color-bar" aria-hidden="true"></div>
				{#if !cellOk[i]}
					<div class="error-indicator" aria-hidden="true">!</div>
				{/if}
				{#if checkedFlags[i]}
					<svg
						class="mark"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2.5"
						stroke-linecap="round"
						stroke-linejoin="round"
						aria-hidden="true"
					>
						<path d="M18 6L6 18M6 6l12 12" />
					</svg>
				{/if}
			</button>
		</div>
	{/each}
</div>

<style>
	.matrix {
		display: grid;
		grid-template-columns: repeat(5, 100px);
		gap: 10px;
		justify-content: start;
		background: transparent;
		padding: 0;
		border-radius: 0;
		box-shadow: none;
	}

	.cell-wrapper {
		position: relative;
	}

	@media (max-width: 720px) {
		.matrix {
			grid-template-columns: repeat(5, 60px);
			gap: 6px;
			padding: 0;
		}
	}

	.cell {
		position: relative;
		width: 100px;
		height: 100px;
		border-radius: 10px;
		border: 1px solid var(--border);
		background: var(--panel);
		box-shadow: var(--inset-highlight);
		cursor: pointer;
		display: grid;
		place-items: center;
		color: var(--text);
		overflow: hidden;
		transition:
			background-color 140ms ease,
			border-color 140ms ease,
			box-shadow 140ms ease,
			transform 80ms ease;
	}

	.cell:hover:not(:disabled) {
		border-color: var(--border-2);
		background: var(--panel-hover);
		box-shadow: var(--inset-highlight);
	}

	.cell:active:not(:disabled) {
		transform: translateY(1px);
		box-shadow: var(--inset-shadow);
	}

	@media (max-width: 720px) {
		.cell {
			width: 60px;
			height: 60px;
			border-radius: 8px;
		}
	}

	.cell:disabled {
		cursor: not-allowed;
		opacity: 0.8;
		background: var(--bg-2);
		border-color: var(--border);
	}

	.color-bar {
		position: absolute;
		top: 0;
		left: 0;
		bottom: 0;
		width: 6px;
		background: var(--cell-color, #94a3b8);
		opacity: 0.95;
		transition: opacity 0.2s;
	}

	/* Subtle background tint based on color */
	.cell::before {
		content: '';
		position: absolute;
		inset: 0;
		background: var(--cell-color);
		opacity: 0.06;
		pointer-events: none;
		transition: opacity 0.2s;
	}

	.cell.checked {
		background: var(--panel);
		color: var(--text);
		border-color: var(--border-2);
		box-shadow: var(--inset-highlight);
	}

	/* Keep the tint visible but maybe slightly adjusted if needed,
	   or just let it sit on top of slate-50 */
	.cell.checked::before {
		opacity: 0.12;
	}
	
	.cell.checked:hover:not(:disabled) {
		border-color: var(--border-2);
	}
	
	.cell.checked .color-bar {
		opacity: 1;
	}

	.mark {
		width: 40px;
		height: 40px;
		opacity: 1;
		color: currentColor;
		position: relative;
		z-index: 2;
		/* filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.1)); */
	}

	@media (max-width: 720px) {
		.mark {
			width: 32px;
			height: 32px;
		}
	}

	.cell.invalid {
		/* border-color: var(--danger); */
		animation: shake 240ms ease-in-out;
		outline: 2px solid var(--danger);
		outline-offset: 2px;
		z-index: 10;
	}

	.error-indicator {
		position: absolute;
		top: 4px;
		right: 4px;
		width: 16px;
		height: 16px;
		background: var(--danger);
		color: white;
		font-size: 10px;
		font-weight: bold;
		border-radius: 50%;
		display: grid;
		place-items: center;
		box-shadow: 0 1px 2px rgb(0 0 0 / 0.2);
	}

	@keyframes shake {
		0% {
			transform: translateX(0);
		}
		25% {
			transform: translateX(-2px);
		}
		50% {
			transform: translateX(2px);
		}
		75% {
			transform: translateX(-1px);
		}
		100% {
			transform: translateX(0);
		}
	}
</style>
