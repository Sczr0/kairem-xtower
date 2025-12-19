<svelte:options runes={false} />

<script lang="ts">
	import { colorToCss, Color, type ColorId } from '$lib/colors';

	const indices = Array.from({ length: 25 }, (_, i) => i);

	export let grid: ColorId[] = Array.from({ length: 25 }, () => Color.White);
	export let checkedMask = 0;
	export let cellOk: boolean[] = Array.from({ length: 25 }, () => true);
	export let cellMessages: (string | undefined)[] = Array.from({ length: 25 }, () => undefined);
	export let onToggle: (index: number) => void = () => {};
	export let onHover: (index: number | null) => void = () => {};

	// 注意：不要把 checkedMask 依赖藏在函数闭包里，否则 Svelte 可能不会在 mask 更新时重算模板。
	// 这里用响应式语句显式建立依赖关系，保证勾选样式能即时更新。
	$: checkedFlags = indices.map((i) => ((checkedMask >>> 0) & (1 << i)) !== 0);
	$: blackFlags = indices.map((i) => grid[i] === Color.Black);

	let hoveredIndex: number | null = null;
	let tooltipVisible = false;
	let tooltipTimer: ReturnType<typeof setTimeout> | null = null;

	function handleMouseEnter(i: number) {
		hoveredIndex = i;
		onHover(i);
		
		// 如果有错误信息，延迟显示 Tooltip
		if (!cellOk[i] && cellMessages[i]) {
			tooltipTimer = setTimeout(() => {
				if (hoveredIndex === i) {
					tooltipVisible = true;
				}
			}, 500); // 500ms 延迟
		}
	}

	function handleMouseLeave() {
		hoveredIndex = null;
		onHover(null);
		tooltipVisible = false;
		if (tooltipTimer) {
			clearTimeout(tooltipTimer);
			tooltipTimer = null;
		}
	}
</script>

<div class="matrix shadow-xl" role="grid" aria-label="5x5 矩阵">
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
			{#if hoveredIndex === i && tooltipVisible && cellMessages[i]}
				<div class="tooltip" role="tooltip">
					{cellMessages[i]}
					<div class="tooltip-arrow"></div>
				</div>
			{/if}
		</div>
	{/each}
</div>

<style>
	.matrix {
		display: grid;
		grid-template-columns: repeat(5, 100px);
		gap: 10px;
		justify-content: start;
		background: #f1f5f9;
		padding: 20px;
		border-radius: 24px;
		box-shadow:
			0 20px 25px -5px rgb(0 0 0 / 0.1),
			0 8px 10px -6px rgb(0 0 0 / 0.1);
	}

	.cell-wrapper {
		position: relative;
	}

	.tooltip {
		position: absolute;
		bottom: 100%;
		left: 50%;
		transform: translateX(-50%);
		margin-bottom: 8px;
		background: #1e293b;
		color: #f8fafc;
		padding: 8px 12px;
		border-radius: 6px;
		font-size: 0.75rem;
		line-height: 1.4;
		white-space: nowrap;
		z-index: 50;
		pointer-events: none;
		box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
		max-width: 200px;
		white-space: normal;
		text-align: center;
		animation: fadeIn 0.2s ease-out;
	}

	.tooltip-arrow {
		position: absolute;
		top: 100%;
		left: 50%;
		transform: translateX(-50%);
		border-width: 6px;
		border-style: solid;
		border-color: #1e293b transparent transparent transparent;
	}

	@keyframes fadeIn {
		from { opacity: 0; transform: translate(-50%, 4px); }
		to { opacity: 1; transform: translate(-50%, 0); }
	}

	@media (max-width: 720px) {
		.matrix {
			grid-template-columns: repeat(5, 60px);
			gap: 6px;
			padding: 12px;
		}
	}

	.cell {
		position: relative;
		width: 100px;
		height: 100px;
		border-radius: 12px;
		border: 1px solid transparent;
		background-color: #ffffff;
		box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1);
		cursor: pointer;
		display: grid;
		place-items: center;
		color: #0f172a;
		overflow: hidden;
		transition:
			background-color 200ms ease,
			transform 150ms cubic-bezier(0.4, 0, 0.2, 1),
			box-shadow 200ms ease,
			border-color 200ms ease;
	}

	.cell:hover:not(:disabled) {
		transform: translateY(-2px);
		box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -1px rgb(0 0 0 / 0.06);
		border-color: #cbd5e1;
	}

	.cell:active:not(:disabled) {
		transform: scale(0.95);
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
		opacity: 0.9;
	}

	.color-bar {
		position: absolute;
		top: 0;
		left: 0;
		bottom: 0;
		width: 6px;
		background: var(--cell-color, #94a3b8);
		opacity: 0.8;
		transition: opacity 0.2s;
	}

	/* Subtle background tint based on color */
	.cell::before {
		content: '';
		position: absolute;
		inset: 0;
		background: var(--cell-color);
		opacity: 0.05;
		pointer-events: none;
		transition: opacity 0.2s;
	}

	.cell.checked {
		background-color: #f8fafc; /* slate-50 */
		color: #334155; /* slate-700 */
		border-color: #475569; /* slate-600 */
		border-width: 2px;
		transform: scale(1);
	}

	/* Keep the tint visible but maybe slightly adjusted if needed,
	   or just let it sit on top of slate-50 */
	.cell.checked::before {
		opacity: 0.1;
	}
	
	.cell.checked:hover:not(:disabled) {
		background-color: #f1f5f9; /* slate-100 */
		border-color: #334155; /* slate-700 */
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
		outline: 2px solid #ef4444;
		outline-offset: 2px;
		z-index: 10;
	}

	.error-indicator {
		position: absolute;
		top: 4px;
		right: 4px;
		width: 16px;
		height: 16px;
		background: #ef4444;
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
