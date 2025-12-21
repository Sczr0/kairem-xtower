<svelte:options runes={false} />

<script lang="ts">
	import Matrix from '$lib/components/Matrix.svelte';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import A11yToggle from '$lib/components/A11yToggle.svelte';
	import { Color, type ColorId, colorToCss } from '$lib/colors';
	import { decodeLevel, encodeLevel, levelToJson, normalizeLevelJson } from '$lib/level-code.js';
	import { browser } from '$app/environment';
	import { onMount } from 'svelte';
	import { slide } from 'svelte/transition';
	import { colorBlindEnabled } from '$lib/a11y';
	import {
		loadEngine,
		type DifficultyReport,
		type Engine,
		type SolutionCountResult,
		type ValidateResult
	} from '$lib/wasm/load';

	let engine: Engine | null = null;
	let engineError = '';

	let grid: ColorId[] = Array.from({ length: 25 }, () => Color.White);
	let selectedColor: ColorId = Color.Black;

	let validate: ValidateResult | null = null;
	let difficulty: DifficultyReport | null = null;
	let solutionCount: SolutionCountResult | null = null;
	let analysisError = '';

	let shareToast = '';
	let shareManualVisible = false;
	let shareUrlForManualCopy = '';

	let importText = '';
	let exportJsonText = '';
	let exportLevelCode = '';
	let playUrl = '';

	const palette: { id: ColorId; label: string }[] = [
		{ id: Color.Black, label: '黑' },
		{ id: Color.White, label: '白' },
		{ id: Color.Red, label: '红' },
		{ id: Color.Blue, label: '蓝' },
		{ id: Color.Green, label: '绿' },
		{ id: Color.Yellow, label: '黄' },
		{ id: Color.Purple, label: '紫' },
		{ id: Color.Orange, label: '橙' },
		{ id: Color.Cyan, label: '青' }
	];

	function showToast(msg: string) {
		shareToast = msg;
		setTimeout(() => {
			if (shareToast === msg) shareToast = '';
		}, 2000);
	}

	function selectAll(event: Event) {
		const input = event.currentTarget as HTMLInputElement | HTMLTextAreaElement | null;
		input?.select();
	}

	function blackMaskFromGrid(flat: ColorId[]): number {
		let m = 0;
		for (let i = 0; i < flat.length; i++) {
			if (flat[i] === Color.Black) m |= 1 << i;
		}
		return m >>> 0;
	}

	function setCell(i: number, color: ColorId) {
		grid[i] = color;
		grid = [...grid];
		scheduleAnalyze();
	}

	function paint(i: number) {
		setCell(i, selectedColor);
	}

	function altPaint(i: number) {
		setCell(i, grid[i] === Color.Black ? Color.White : Color.Black);
	}

	let analyzeTimer: ReturnType<typeof setTimeout> | null = null;
	function scheduleAnalyze() {
		if (!engine) return;
		if (analyzeTimer) clearTimeout(analyzeTimer);
		analyzeTimer = setTimeout(() => {
			analyzeTimer = null;
			runAnalyze();
		}, 120);
	}

	function refreshExports() {
		exportJsonText = JSON.stringify(levelToJson(grid), null, 2);
		try {
			exportLevelCode = encodeLevel(grid);
		} catch {
			exportLevelCode = '';
		}

		if (browser && exportLevelCode) {
			const url = new URL(window.location.origin);
			url.searchParams.set('level', exportLevelCode);
			playUrl = url.toString();
		} else {
			playUrl = '';
		}
	}

	function runAnalyze() {
		if (!engine) return;
		analysisError = '';

		try {
			const blackMask = blackMaskFromGrid(grid);
			validate = engine.validate_state(blackMask, new Uint8Array(grid));
		} catch {
			validate = null;
		}

		try {
			difficulty = engine.difficulty_report(new Uint8Array(grid));
		} catch {
			difficulty = null;
		}

		try {
			solutionCount = engine.solution_count(new Uint8Array(grid), 2);
		} catch (e) {
			solutionCount = null;
			analysisError = String(e);
		}

		refreshExports();
	}

	function solutionLabel(v: SolutionCountResult | null): string {
		if (!v) return '--';
		if (v.count === 0) return '无解';
		if (v.count === 1) return '唯一解';
		return '多解（>=2）';
	}

	function parseImportToGrid(text: string): ColorId[] {
		const raw = text.trim();
		if (!raw) throw new Error('导入内容为空');

		// 支持粘贴完整 URL：.../?level=xxxx
		if (raw.includes('level=')) {
			const url = browser ? new URL(raw, window.location.href) : new URL(raw);
			const level = url.searchParams.get('level');
			if (!level) throw new Error('URL 中未找到 level 参数');
			const decoded = decodeLevel(level);
			return decoded.grid as ColorId[];
		}

		// JSON
		if (raw.startsWith('{') || raw.startsWith('[')) {
			const value = JSON.parse(raw);
			const flat = normalizeLevelJson(value);
			return flat as ColorId[];
		}

		// level code
		const decoded = decodeLevel(raw);
		return decoded.grid as ColorId[];
	}

	function applyImport() {
		try {
			const next = parseImportToGrid(importText);
			if (next.length !== 25) throw new Error('导入关卡长度非法');
			grid = next;
			showToast('已导入');
			scheduleAnalyze();
		} catch (e) {
			showToast(`导入失败：${String(e)}`);
		}
	}

	async function copyPlayUrl() {
		if (!browser || !playUrl) return;
		shareUrlForManualCopy = playUrl;
		shareManualVisible = false;

		if (typeof navigator.share === 'function') {
			try {
				await navigator.share({ title: '关卡试玩链接', url: playUrl });
				showToast('已打开系统分享');
				return;
			} catch {
				// 用户取消也会抛错，继续走复制兜底即可
			}
		}

		if (typeof navigator.clipboard?.writeText !== 'function') {
			shareManualVisible = true;
			showToast('当前环境不支持自动复制，请手动复制链接');
			return;
		}

		try {
			await navigator.clipboard.writeText(playUrl);
			showToast('链接已复制');
		} catch {
			shareManualVisible = true;
			showToast('复制失败，请手动复制链接');
		}
	}

	function setFromUrlLevelParam(level: string) {
		try {
			const decoded = decodeLevel(level);
			grid = decoded.grid as ColorId[];
			showToast('已从链接加载关卡');
		} catch (e) {
			showToast(`level 参数无效：${String(e)}`);
		}
	}

	onMount(async () => {
		try {
			engine = await loadEngine();

			if (browser) {
				const level = new URL(window.location.href).searchParams.get('level');
				if (level) setFromUrlLevelParam(level);
			}

			runAnalyze();
		} catch (e) {
			engineError = String(e);
		}
	});
</script>

<div class="page">
	<header class="header">
		<div class="header-left">
			<a class="btn btn-ghost back" href="/">返回游戏</a>
			<div class="title-area">
				<h1 class="title">关卡编辑器</h1>
				<p class="subtitle">导入/导出 + 解唯一性检测（limit=2）</p>
			</div>
		</div>
		<div class="header-actions">
			<a class="btn" href="/daily">日历</a>
			<A11yToggle />
			<ThemeToggle />
		</div>
	</header>

	{#if shareToast}
		<div class="toast" transition:slide={{ axis: 'y' }}>{shareToast}</div>
	{/if}

	{#if shareManualVisible && shareUrlForManualCopy}
		<div class="share-manual" transition:slide>
			<input class="input" readonly value={shareUrlForManualCopy} on:focus={selectAll} on:click={selectAll} />
		</div>
	{/if}

	{#if engineError}
		<div class="error-banner">引擎加载失败：{engineError}</div>
	{:else}
		<div class="layout">
			<main class="card editor-area">
				<div class="section">
					<div class="section-head">
						<div>
							<div class="section-title">画笔</div>
							<div class="section-hint">左键：涂色；右键：黑/白快速切换</div>
						</div>
					</div>

					<div class="palette">
						{#each palette as p}
							<button
								type="button"
								class="swatch {selectedColor === p.id ? 'active' : ''}"
								style="--swatch-color: {colorToCss(p.id)}"
								on:click={() => (selectedColor = p.id)}
								title={p.label}
							>
								<span class="swatch-dot" aria-hidden="true"></span>
								<span class="swatch-label">{p.label}</span>
							</button>
						{/each}
					</div>
				</div>

				<div class="matrix-wrap">
					<Matrix
						mode="edit"
						grid={grid}
						checkedMask={0}
						colorBlindMode={$colorBlindEnabled}
						cellOk={validate?.cell_ok ?? Array.from({ length: 25 }, () => true)}
						onPaint={paint}
						onAltPaint={altPaint}
					/>
				</div>

				<div class="section">
					<div class="section-head">
						<div>
							<div class="section-title">导入</div>
							<div class="section-hint">支持：关卡 JSON / level 编码 / 完整 URL（含 ?level=）</div>
						</div>
						<button class="btn btn-primary" type="button" on:click={applyImport} disabled={!importText.trim()}>
							导入
						</button>
					</div>

					<textarea
						class="textarea"
						rows="5"
						bind:value={importText}
						placeholder="粘贴关卡 JSON、level 编码，或完整 URL..."
					></textarea>
				</div>
			</main>

			<aside class="side">
				<div class="card panel">
					<div class="panel-title">质量检查</div>

					<div class="kv">
						<div class="k">解唯一性</div>
						<div class="v">
							<span class="chip">{solutionLabel(solutionCount)}</span>
						</div>
					</div>

					<div class="kv">
						<div class="k">难度</div>
						<div class="v">
							<span class="chip">{difficulty ? difficulty.difficulty_score : '--'}</span>
						</div>
					</div>

					{#if analysisError}
						<div class="hint-banner">分析失败：{analysisError}</div>
					{/if}

					<div class="panel-actions">
						<button class="btn btn-primary" type="button" on:click={copyPlayUrl} disabled={!playUrl}>
							复制试玩链接
						</button>
						<a
							class="btn {playUrl ? '' : 'btn-disabled'}"
							href={playUrl || '#'}
							target="_blank"
							rel="noopener noreferrer"
							aria-disabled={!playUrl}
							tabindex={playUrl ? 0 : -1}
						>
							打开试玩
						</a>
					</div>
				</div>

				<div class="card panel">
					<div class="panel-title">导出</div>

					<div class="sub-title">level 编码</div>
					<input class="input" readonly value={exportLevelCode} on:focus={selectAll} on:click={selectAll} />

					<div class="sub-title">关卡 JSON</div>
					<textarea
						class="textarea"
						rows="8"
						readonly
						value={exportJsonText}
						on:focus={selectAll}
						on:click={selectAll}
					></textarea>
				</div>
			</aside>
		</div>
	{/if}
</div>

<style>
	.page {
		max-width: 1200px;
		margin: 0 auto;
	}

	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		margin-bottom: 16px;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.back {
		padding-left: 10px;
		padding-right: 10px;
	}

	.title-area {
		display: grid;
		gap: 2px;
	}

	.title {
		margin: 0;
		font-size: 1.25rem;
		font-weight: 900;
		letter-spacing: -0.02em;
	}

	.subtitle {
		margin: 0;
		color: var(--muted);
		font-size: 0.9rem;
	}

	.layout {
		display: grid;
		grid-template-columns: 1fr 360px;
		gap: 16px;
		align-items: start;
	}

	@media (max-width: 980px) {
		.layout {
			grid-template-columns: 1fr;
		}
	}

	.editor-area {
		padding: 16px 16px;
	}

	.matrix-wrap {
		margin: 14px 0 18px;
	}

	.section {
		margin-bottom: 14px;
	}

	.section-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		margin-bottom: 10px;
	}

	.section-title {
		font-weight: 900;
		color: var(--text);
		font-size: 0.98rem;
	}

	.section-hint {
		color: var(--muted);
		font-size: 0.85rem;
		margin-top: 2px;
	}

	.palette {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.swatch {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 8px 10px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border);
		background: var(--panel);
		cursor: pointer;
	}

	.swatch:hover {
		border-color: var(--border-2);
		background: var(--panel-hover);
	}

	.swatch.active {
		border-color: color-mix(in srgb, var(--swatch-color) 55%, var(--border));
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--swatch-color) 22%, transparent);
	}

	.swatch-dot {
		width: 12px;
		height: 12px;
		border-radius: 999px;
		background: var(--swatch-color);
		border: 1px solid color-mix(in srgb, var(--swatch-color) 55%, var(--border));
	}

	.swatch-label {
		font-size: 0.85rem;
		font-weight: 700;
		color: var(--text);
	}

	.textarea {
		width: 100%;
		padding: 10px 10px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--panel);
		color: var(--text);
		font-family: var(--mono);
		resize: vertical;
	}

	.side {
		display: grid;
		gap: 16px;
	}

	.panel {
		padding: 14px 14px;
	}

	.panel-title {
		font-weight: 900;
		margin-bottom: 10px;
	}

	.kv {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		padding: 8px 0;
		border-top: 1px solid var(--border);
	}

	.kv:first-of-type {
		border-top: none;
	}

	.k {
		color: var(--muted);
		font-size: 0.9rem;
	}

	.v {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	.chip {
		border: 1px solid var(--border);
		border-radius: 999px;
		padding: 3px 10px;
		font-size: 0.82rem;
		font-weight: 800;
		background: var(--bg-2);
		color: var(--text);
	}

	.panel-actions {
		display: flex;
		gap: 10px;
		margin-top: 12px;
	}

	.btn-disabled {
		pointer-events: none;
		opacity: 0.55;
	}

	.sub-title {
		margin-top: 12px;
		margin-bottom: 6px;
		color: var(--muted);
		font-size: 0.85rem;
		font-weight: 800;
	}

	.toast {
		background: var(--tooltip-bg);
		border: 1px solid var(--tooltip-border);
		color: var(--tooltip-text);
		padding: 10px 12px;
		border-radius: var(--radius-sm);
		text-align: center;
		font-size: 0.92rem;
		margin-bottom: 12px;
		box-shadow: var(--shadow-soft);
	}

	.share-manual {
		margin-bottom: 12px;
	}

	.hint-banner {
		margin-top: 10px;
		background: var(--danger-surface);
		border: 1px solid var(--danger-border);
		color: var(--danger-text);
		padding: 10px 12px;
		border-radius: var(--radius-sm);
	}

	.error-banner {
		background: var(--danger-surface);
		border: 1px solid var(--danger-border);
		color: var(--danger-text);
		padding: 12px 14px;
		border-radius: var(--radius-lg);
		text-align: center;
	}
</style>
