<svelte:options runes={false} />

<script lang="ts">
	import { colorToCss, Color, type ColorId } from '$lib/colors';

	const indices = Array.from({ length: 25 }, (_, i) => i);

	export let grid: ColorId[] = Array.from({ length: 25 }, () => Color.White);
	export let checkedMask = 0;
	export let marks: number[] = Array.from({ length: 25 }, () => 0);
	export let cellOk: boolean[] = Array.from({ length: 25 }, () => true);
	export let onToggle: (index: number) => void = () => {};
	export let onMarkCycle: (index: number) => void = () => {};
	export let onHover: (index: number | null) => void = () => {};
	// 编辑模式：用于关卡编辑器（不影响玩法页默认行为）
	export let mode: 'play' | 'edit' = 'play';
	export let onPaint: (index: number) => void = () => {};
	export let onAltPaint: (index: number) => void = () => {};
	// 提示高亮：仅用于 UI 引导，不影响逻辑。
	export let hintIndex: number | null = null;
	export let hintAction: 'check' | 'uncheck' | null = null;

	// 注意：不要把 checkedMask 依赖藏在函数闭包里，否则 Svelte 可能不会在 mask 更新时重算模板。
	// 这里用响应式语句显式建立依赖关系，保证勾选样式能即时更新。
	$: checkedFlags = indices.map((i) => ((checkedMask >>> 0) & (1 << i)) !== 0);
	$: blackFlags = indices.map((i) => grid[i] === Color.Black);

	let longPressTimer: ReturnType<typeof setTimeout> | null = null;
	let longPressIndex: number | null = null;
	let longPressTriggered = false;
	let suppressContextMenuUntil = 0;

	function clearLongPress() {
		if (longPressTimer) clearTimeout(longPressTimer);
		longPressTimer = null;
		longPressIndex = null;
		longPressTriggered = false;
	}

	function handleMouseEnter(i: number) {
		onHover(i);
	}

	function handleMouseLeave() {
		onHover(null);
	}

	function handleClick(i: number) {
		if (mode === 'edit') onPaint(i);
		else {
			if (longPressTriggered && longPressIndex === i) {
				clearLongPress();
				return;
			}
			onToggle(i);
		}
	}

	function handleContextMenu(event: MouseEvent, i: number) {
		event.preventDefault();
		if (mode === 'edit') onAltPaint(i);
		else {
			// iOS/Safari çš„é•¿æŒ‰å¯èƒ½ä¼šä¿®å¤æ€§åœ°è§¦å‘ contextmenuï¼Œé¿å…é•¿æŒ‰å¯¼è‡´å¾ªçŽ¯ä¸¤æ¬¡ã€?
			if (Date.now() < suppressContextMenuUntil) return;
			onMarkCycle(i);
		}
	}

	function handlePointerDown(event: PointerEvent, i: number) {
		if (mode !== 'play') return;
		if (event.pointerType !== 'touch') return;
		if (blackFlags[i]) return;

		if (longPressTimer) clearTimeout(longPressTimer);
		longPressIndex = i;
		longPressTriggered = false;
		longPressTimer = setTimeout(() => {
			longPressTriggered = true;
			suppressContextMenuUntil = Date.now() + 1200;
			onMarkCycle(i);
		}, 420);
	}

	function handlePointerUp(event: PointerEvent, i: number) {
		if (mode !== 'play') return;
		if (event.pointerType !== 'touch') return;
		if (longPressIndex !== i) return;

		if (longPressTimer) clearTimeout(longPressTimer);
		longPressTimer = null;
	}

	function handlePointerCancel(event: PointerEvent) {
		if (mode !== 'play') return;
		if (event.pointerType !== 'touch') return;
		clearLongPress();
	}
</script>

<div class="matrix" role="grid" aria-label="5x5 矩阵">
	{#each indices as i}
		<div class="cell-wrapper">
			<button
				type="button"
				class="cell {mode === 'play' && checkedFlags[i] ? 'checked' : ''} {cellOk[i] ? '' : 'invalid'} {mode === 'play' && hintIndex === i ? 'hint' : ''} {mode === 'play' && hintIndex === i && hintAction ? `hint-${hintAction}` : ''}"
				style="--cell-color: {colorToCss(grid[i])}"
				aria-pressed={mode === 'play' ? checkedFlags[i] : undefined}
				disabled={mode === 'play' && blackFlags[i]}
				on:click={() => handleClick(i)}
				on:contextmenu={(e) => handleContextMenu(e, i)}
				on:pointerdown={(e) => handlePointerDown(e, i)}
				on:pointerup={(e) => handlePointerUp(e, i)}
				on:pointercancel={handlePointerCancel}
				on:mouseenter={() => handleMouseEnter(i)}
				on:mouseleave={handleMouseLeave}
				on:focus={() => handleMouseEnter(i)}
				on:blur={handleMouseLeave}
			>
				<div class="color-bar" aria-hidden="true"></div>
				{#if !cellOk[i]}
					<div class="error-indicator" aria-hidden="true">!</div>
				{/if}
				{#if mode === 'play' && checkedFlags[i]}
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
				{#if mode === 'play' && marks[i] === 1}
					<div class="note note-exclude" aria-hidden="true">⊘</div>
				{:else if mode === 'play' && marks[i] === 2}
					<div class="note note-question" aria-hidden="true">?</div>
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
		touch-action: manipulation;
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

	.note {
		position: absolute;
		top: 6px;
		left: 8px;
		font-size: 14px;
		font-weight: 700;
		line-height: 1;
		padding: 2px 6px;
		border-radius: 999px;
		background: color-mix(in srgb, var(--panel) 60%, transparent);
		border: 1px solid var(--border);
		color: var(--text);
		z-index: 3;
		pointer-events: none;
	}

	.note-exclude {
		color: var(--danger);
	}

	.note-question {
		color: var(--c-blue);
	}

	@media (max-width: 720px) {
		.note {
			top: 4px;
			left: 6px;
			font-size: 12px;
			padding: 1px 5px;
		}
	}

	.cell.invalid {
		/* border-color: var(--danger); */
		animation: shake 240ms ease-in-out;
		outline: 2px solid var(--danger);
		outline-offset: 2px;
		z-index: 10;
	}

	.cell.hint {
		/* 用 ring 而不是 outline，避免与 invalid 的 outline 冲突 */
		z-index: 9;
		box-shadow:
			0 0 0 3px color-mix(in srgb, var(--c-blue) 45%, transparent),
			var(--inset-highlight);
	}

	.cell.hint-check {
		box-shadow:
			0 0 0 3px color-mix(in srgb, var(--success) 45%, transparent),
			var(--inset-highlight);
	}

	.cell.hint-uncheck {
		box-shadow:
			0 0 0 3px color-mix(in srgb, var(--danger) 45%, transparent),
			var(--inset-highlight);
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
