<svelte:options runes={false} />

<script lang="ts">
	import Matrix from '$lib/components/Matrix.svelte';
	import RuleCard, { type Rule as UiRule } from '$lib/components/RuleCard.svelte';
	import rules from '$lib/rules.json';
	import { Color, type ColorId, colorToCss } from '$lib/colors';
	import { loadEngine, type Engine, type ValidateResult } from '$lib/wasm/load';
	import { browser, dev } from '$app/environment';
	import { onMount } from 'svelte';

	let engine: Engine | null = null;
	let engineError = '';

	let puzzleKind: 'daily' | 'seed' = 'daily';
	let urlSeedError = '';

	let dateYmd = '';
	let seed: bigint | null = null;

	let shareToast = '';
	let shareUrlForManualCopy = '';
	let shareManualVisible = false;

	let grid2d: number[][] = [];
	let grid: ColorId[] = Array.from({ length: 25 }, () => Color.White);
	let checkedMask = 0;
	let validate: ValidateResult | null = null;
	let hoveredRuleId: string | null = null;

	type Rule = UiRule;
	const allRules = (rules.rules ?? []) as Rule[];
	const goalRule = allRules.find((r) => r.id === 'bingo') ?? null;

	const ruleColorMap: Partial<Record<string, ColorId>> = {
		red: Color.Red,
		blue: Color.Blue,
		green: Color.Green,
		yellow: Color.Yellow,
		purple: Color.Purple,
		orange: Color.Orange,
		cyan: Color.Cyan,
		black: Color.Black
	};

	// 反向映射：ColorId -> RuleId
	const colorRuleMap = Object.entries(ruleColorMap).reduce(
		(acc, [ruleId, colorId]) => {
			if (colorId !== undefined) {
				acc[colorId] = ruleId;
			}
			return acc;
		},
		{} as Record<number, string>
	);

	function ruleColorCss(ruleId: string): string {
		if (ruleId === 'bingo') return '#f59e0b';
		const colorId = ruleColorMap[ruleId];
		if (colorId === undefined) return '#94a3b8';
		return colorToCss(colorId);
	}

	$: hoveredRule = hoveredRuleId ? allRules.find((r) => r.id === hoveredRuleId) ?? null : null;

	function shanghaiDateYmd(now = new Date()): string {
		const fmt = new Intl.DateTimeFormat('zh-CN', {
			timeZone: 'Asia/Shanghai',
			year: 'numeric',
			month: '2-digit',
			day: '2-digit'
		});
		const parts = fmt.formatToParts(now);
		const y = parts.find((p) => p.type === 'year')?.value ?? '1970';
		const m = parts.find((p) => p.type === 'month')?.value ?? '01';
		const d = parts.find((p) => p.type === 'day')?.value ?? '01';
		return `${y}-${m}-${d}`;
	}

	function parseSeed(raw: string): bigint | null {
		const s = raw.trim();
		if (!s) return null;
		try {
			const v = BigInt(s);
			if (v < 0n) return null;
			return v;
		} catch {
			return null;
		}
	}

	function shortSeed(v: bigint): string {
		const s = v.toString();
		if (s.length <= 12) return s;
		return `${s.slice(0, 6)}…${s.slice(-4)}`;
	}

	function buildSeedUrl(v: bigint): string {
		if (!browser) return '';
		const url = new URL(window.location.href);
		url.searchParams.set('seed', v.toString());
		return url.toString();
	}

	function replaceUrlSeed(v: bigint | null) {
		if (!browser) return;
		const url = new URL(window.location.href);
		if (v === null) url.searchParams.delete('seed');
		else url.searchParams.set('seed', v.toString());
		window.history.replaceState({}, '', url.toString());
	}

	function randomSeedU64(): bigint {
		if (browser && typeof window.crypto?.getRandomValues === 'function') {
			const buf = new Uint32Array(2);
			window.crypto.getRandomValues(buf);
			return (BigInt(buf[0]) << 32n) | BigInt(buf[1]);
		}
		const hi = Math.floor(Math.random() * 0x1_0000_0000);
		const lo = Math.floor(Math.random() * 0x1_0000_0000);
		return (BigInt(hi) << 32n) | BigInt(lo);
	}

	function showToast(msg: string) {
		shareToast = msg;
		setTimeout(() => {
			if (shareToast === msg) shareToast = '';
		}, 2000);
	}

	function selectAll(event: Event) {
		const input = event.currentTarget as HTMLInputElement | null;
		input?.select();
	}

	function flattenGrid2d(g: number[][]): ColorId[] {
		return g.flat().map((x) => x as ColorId);
	}

	function blackMaskFromGrid(flat: ColorId[]): number {
		let m = 0;
		for (let i = 0; i < flat.length; i++) {
			if (flat[i] === Color.Black) m |= 1 << i;
		}
		return m >>> 0;
	}

	function refreshValidate() {
		if (!engine) return;
		validate = engine.validate_state(checkedMask >>> 0, new Uint8Array(grid));
	}

	function focusRuleByIndex(i: number) {
		const color = grid[i];
		hoveredRuleId = colorRuleMap[color] ?? null;
	}

	function toggle(i: number) {
		if (grid[i] === Color.Black) return;
		checkedMask = (checkedMask ^ (1 << i)) >>> 0;
		focusRuleByIndex(i);
		refreshValidate();
	}

	function handleHover(index: number | null) {
		if (index === null) {
			hoveredRuleId = null;
			return;
		}
		focusRuleByIndex(index);
	}

	async function loadPuzzleBySeed(newSeed: bigint, opts: { updateUrl?: boolean } = {}) {
		if (!engine) return;
		seed = newSeed;
		grid2d = engine.generate_puzzle(seed);
		grid = flattenGrid2d(grid2d);
		checkedMask = blackMaskFromGrid(grid);
		hoveredRuleId = null;
		refreshValidate();
		if (opts.updateUrl) replaceUrlSeed(seed);
	}

	async function newSeedPuzzle(newSeed: bigint, opts: { updateUrl?: boolean } = {}) {
		if (!engine) return;
		puzzleKind = 'seed';
		urlSeedError = '';
		dateYmd = '';
		await loadPuzzleBySeed(newSeed, opts);
	}

	async function newDailyPuzzle(opts: { updateUrl?: boolean; keepUrlSeedError?: boolean } = {}) {
		if (!engine) return;
		puzzleKind = 'daily';
		if (!opts.keepUrlSeedError) urlSeedError = '';
		dateYmd = shanghaiDateYmd();
		const dailySeed = engine.date_to_seed_ymd(dateYmd);
		await loadPuzzleBySeed(dailySeed);
		if (opts.updateUrl) replaceUrlSeed(null);
	}

	async function newRandomPuzzle() {
		if (!engine) return;
		await newSeedPuzzle(randomSeedU64(), { updateUrl: true });
	}

	async function sharePuzzle() {
		if (!seed || !browser) return;

		const url = buildSeedUrl(seed);
		shareUrlForManualCopy = url;
		shareManualVisible = false;

		if (typeof navigator.share === 'function') {
			try {
				await navigator.share({ title: 'Kairem', url });
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
			await navigator.clipboard.writeText(url);
			showToast('链接已复制');
		} catch {
			shareManualVisible = true;
			showToast('复制失败，请手动复制链接');
		}
	}

	onMount(async () => {
		try {
			engine = await loadEngine();

			const rawSeed = new URL(window.location.href).searchParams.get('seed');
			if (rawSeed !== null) {
				const parsed = parseSeed(rawSeed);
				if (parsed === null) {
					urlSeedError = 'seed 参数无效，已回退到今日题目';
					await newDailyPuzzle({ updateUrl: true, keepUrlSeedError: true });
					return;
				}

				await newSeedPuzzle(parsed, { updateUrl: true });
				return;
			}

			await newDailyPuzzle();
		} catch (e) {
			engineError = String(e);
		}
	});
</script>

<div class="kairem-page">
	<header class="hero">
		<h1 class="hero-title">Kairem</h1>
		<p class="hero-subtitle">无需服务器交互的轻量级逻辑游戏</p>
	</header>

	{#if engineError}
		<div class="error-banner">引擎加载失败：{engineError}</div>
	{:else}
		<section class="layout">
			<div class="left">
				<div class="topbar">
					<div class="daily">
						<span class="daily-label">{puzzleKind === 'daily' ? '今日题目' : '分享题目'}</span>
						{#if puzzleKind === 'daily'}
							<span class="daily-value">{dateYmd || '—'}</span>
						{:else}
							<span class="daily-value" title={seed?.toString() ?? ''}>
								{seed ? shortSeed(seed) : '—'}
							</span>
						{/if}
					</div>

					<div class="actions">
						<button type="button" class="action primary" on:click={sharePuzzle} disabled={!seed}>
							分享
						</button>
						<button type="button" class="action" on:click={newRandomPuzzle} disabled={!engine}>
							随机一题
						</button>
						{#if puzzleKind !== 'daily'}
							<button
								type="button"
								class="action ghost"
								on:click={() => newDailyPuzzle({ updateUrl: true })}
								disabled={!engine}
							>
								回到今日
							</button>
						{/if}
					</div>
				</div>

				{#if shareToast}
					<div class="toast" role="status" aria-live="polite">{shareToast}</div>
				{/if}

				{#if shareManualVisible && shareUrlForManualCopy}
					<div class="share-manual" aria-label="分享链接">
						<input
							class="share-input"
							readonly
							value={shareUrlForManualCopy}
							on:focus={selectAll}
							on:click={selectAll}
						/>
					</div>
				{/if}

				{#if urlSeedError}
					<div class="hint-banner" role="status" aria-live="polite">{urlSeedError}</div>
				{/if}

				<div class="help-card" aria-label="新手引导">
					<h2 class="help-title">怎么玩</h2>
					<ul class="help-list">
						<li>点击非黑格：切换勾选/取消。</li>
						<li>黑格必须勾选（已锁定）。</li>
						<li>出现红框/叹号：悬停格子看原因，并在右侧定位对应规则。</li>
					</ul>
				</div>

				<div class="matrix-shell">
					<Matrix
						grid={grid}
						checkedMask={checkedMask}
						cellOk={validate?.cell_ok ?? Array(25).fill(true)}
						cellMessages={validate?.cell_messages ?? Array(25).fill(undefined)}
						onToggle={toggle}
						onHover={handleHover}
					/>
				</div>

				{#if validate}
					<div class="status-row" aria-label="状态">
						<span class="status {validate.is_valid ? 'ok' : 'warn'}">约束：{validate.is_valid ? '已满足' : '未满足'}</span>
						<span class="status {validate.is_bingo ? 'ok' : ''}">目标：{validate.is_bingo ? '已达成' : '未达成'}</span>
					</div>
				{/if}

				{#if dev}
					<details class="debug">
						<summary>调试信息</summary>
						<div class="debug-body">
							<span class="debug-chip">seed: {seed?.toString() ?? '—'}</span>
							<span class="debug-chip">mask: {checkedMask >>> 0}</span>
							{#if validate}
								<span class="debug-chip">valid: {validate.is_valid}</span>
								<span class="debug-chip">bingo: {validate.is_bingo}</span>
							{/if}
						</div>
					</details>
				{/if}
			</div>

			<aside class="right">
				<div class="rules-panel" aria-label="规则面板">
					<div class="panel-header">
						<h2 class="panel-title">规则</h2>
						<p class="panel-hint">提示：悬停（或点击）格子可定位规则；红色叹号表示当前违反。</p>
						<p class="panel-hint">标签：约束=始终生效；仅勾选时=只有被勾选时生效；目标=通关条件。</p>
					</div>

					<div class="panel-section">
						<div class="section-title">当前关注</div>
						{#if hoveredRule}
							<RuleCard rule={hoveredRule} color={ruleColorCss(hoveredRule.id)} highlighted />
						{:else}
							<div class="empty-state">悬停（或点击）一个格子，以查看它对应的规则。</div>
						{/if}
					</div>

					{#if goalRule}
						<div class="panel-section">
							<div class="section-title">目标</div>
							<RuleCard rule={goalRule} color={ruleColorCss(goalRule.id)} />
						</div>
					{/if}

					<details class="all-rules">
						<summary>
							全部规则 <span class="count">{allRules.length}</span>
						</summary>
						<div class="rules-list">
							{#each allRules as r}
								<RuleCard rule={r} color={ruleColorCss(r.id)} highlighted={hoveredRuleId === r.id} />
							{/each}
						</div>
					</details>
				</div>
			</aside>
		</section>
	{/if}
</div>

<style>
	.kairem-page {
		max-width: 1040px;
		margin: 0 auto;
		padding: 24px 16px 40px;
	}

	.hero {
		text-align: center;
		margin-bottom: 18px;
	}

	.hero-title {
		font-size: 2.6rem;
		font-weight: 850;
		letter-spacing: -0.06em;
		margin: 0;
		background: linear-gradient(135deg, #0f172a 0%, #334155 100%);
		background-clip: text;
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
	}

	.hero-subtitle {
		margin: 8px 0 0;
		color: var(--muted);
		font-size: 1.05rem;
		line-height: 1.5;
	}

	.layout {
		display: grid;
		grid-template-columns: 1fr;
		gap: 22px;
		align-items: start;
	}

	@media (min-width: 980px) {
		.layout {
			grid-template-columns: minmax(0, 1fr) 360px;
		}

		.right {
			position: sticky;
			top: 24px;
		}
	}

	.topbar {
		display: flex;
		justify-content: center;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
		margin-bottom: 14px;
	}

	@media (min-width: 980px) {
		.topbar {
			justify-content: flex-start;
		}
	}

	.daily {
		display: inline-flex;
		align-items: baseline;
		gap: 10px;
		padding: 6px 12px;
		border-radius: 999px;
		background: rgba(241, 245, 249, 0.9);
		border: 1px solid var(--border);
		box-shadow: 0 1px 2px rgb(15 23 42 / 0.04);
	}

	.daily-label {
		color: #94a3b8;
		font-size: 0.72rem;
		letter-spacing: 0.08em;
		font-weight: 900;
	}

	.daily-value {
		font-family: var(--mono);
		font-size: 0.9rem;
		font-weight: 750;
		color: #334155;
	}

	.actions {
		display: inline-flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		justify-content: center;
	}

	.action {
		border: 1px solid var(--border);
		background: rgba(255, 255, 255, 0.9);
		border-radius: 999px;
		padding: 6px 12px;
		font-size: 0.82rem;
		font-weight: 900;
		color: #334155;
		cursor: pointer;
		box-shadow: 0 1px 2px rgb(15 23 42 / 0.04);
		transition:
			transform 120ms ease,
			background 120ms ease,
			border-color 120ms ease;
	}

	.action:hover:not(:disabled) {
		transform: translateY(-1px);
		background: rgba(241, 245, 249, 0.95);
		border-color: rgba(100, 116, 139, 0.35);
	}

	.action:disabled {
		opacity: 0.55;
		cursor: not-allowed;
	}

	.action.primary {
		background: rgba(15, 23, 42, 0.92);
		color: #f8fafc;
		border-color: rgba(15, 23, 42, 0.9);
	}

	.action.primary:hover:not(:disabled) {
		background: rgba(15, 23, 42, 1);
		border-color: rgba(15, 23, 42, 1);
	}

	.action.ghost {
		background: transparent;
		border-style: dashed;
	}

	.toast {
		margin: 0 auto 12px;
		max-width: 720px;
		border-radius: 12px;
		padding: 10px 12px;
		border: 1px solid rgba(148, 163, 184, 0.28);
		background: rgba(241, 245, 249, 0.9);
		color: #334155;
		font-size: 0.85rem;
		font-weight: 800;
		text-align: center;
	}

	.share-manual {
		margin: 0 auto 12px;
		max-width: 840px;
	}

	.share-input {
		width: 100%;
		padding: 10px 12px;
		border-radius: 12px;
		border: 1px solid var(--border);
		background: rgba(255, 255, 255, 0.9);
		box-shadow: var(--shadow);
		font-family: var(--mono);
		font-size: 0.82rem;
		color: #0f172a;
	}

	.hint-banner {
		margin: 0 auto 12px;
		max-width: 720px;
		border-radius: 12px;
		padding: 10px 12px;
		border: 1px solid rgba(59, 130, 246, 0.2);
		background: rgba(219, 234, 254, 0.7);
		color: #1d4ed8;
		font-size: 0.85rem;
		font-weight: 800;
		text-align: center;
	}

	.help-card {
		background: rgba(255, 255, 255, 0.9);
		border: 1px solid var(--border);
		border-radius: 16px;
		padding: 14px 16px;
		box-shadow: var(--shadow);
		margin-bottom: 14px;
	}

	.matrix-shell {
		display: flex;
		justify-content: center;
	}

	@media (min-width: 980px) {
		.matrix-shell {
			justify-content: flex-start;
		}
	}

	.help-title {
		margin: 0 0 10px;
		font-size: 0.9rem;
		font-weight: 900;
		letter-spacing: -0.02em;
		color: #0f172a;
	}

	.help-list {
		margin: 0;
		padding-left: 18px;
		color: var(--muted);
		line-height: 1.6;
		font-size: 0.9rem;
	}

	.help-list li {
		margin: 4px 0;
	}

	.status-row {
		margin-top: 12px;
		display: flex;
		gap: 10px;
		flex-wrap: wrap;
		justify-content: center;
	}

	@media (min-width: 980px) {
		.status-row {
			justify-content: flex-start;
		}
	}

	.status {
		font-size: 0.82rem;
		font-weight: 800;
		border-radius: 999px;
		padding: 4px 10px;
		border: 1px solid rgba(148, 163, 184, 0.25);
		background: rgba(148, 163, 184, 0.12);
		color: #334155;
	}

	.status.ok {
		background: rgba(34, 197, 94, 0.12);
		border-color: rgba(34, 197, 94, 0.18);
		color: #166534;
	}

	.status.warn {
		background: rgba(239, 68, 68, 0.12);
		border-color: rgba(239, 68, 68, 0.18);
		color: #991b1b;
	}

	.debug {
		margin-top: 14px;
	}

	.debug summary {
		cursor: pointer;
		user-select: none;
		color: var(--muted);
		font-size: 0.85rem;
	}

	.debug-body {
		margin-top: 10px;
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}

	.debug-chip {
		font-family: var(--mono);
		font-size: 0.75rem;
		background: rgba(241, 245, 249, 0.9);
		border: 1px solid var(--border);
		border-radius: 999px;
		padding: 3px 8px;
		color: #334155;
	}

	.rules-panel {
		background: rgba(255, 255, 255, 0.9);
		border: 1px solid var(--border);
		border-radius: 16px;
		padding: 16px;
		box-shadow: var(--shadow);
	}

	.panel-header {
		margin-bottom: 12px;
	}

	.panel-title {
		margin: 0;
		font-size: 1rem;
		font-weight: 950;
		letter-spacing: -0.02em;
		color: #0f172a;
	}

	.panel-hint {
		margin: 8px 0 0;
		color: var(--muted);
		font-size: 0.85rem;
		line-height: 1.4;
	}

	.panel-section {
		margin-top: 14px;
		display: grid;
		gap: 10px;
	}

	.section-title {
		font-size: 0.72rem;
		letter-spacing: 0.08em;
		color: #94a3b8;
		font-weight: 950;
		text-transform: uppercase;
	}

	.empty-state {
		font-size: 0.85rem;
		color: var(--muted);
		background: rgba(241, 245, 249, 0.8);
		border: 1px dashed rgba(148, 163, 184, 0.4);
		border-radius: 12px;
		padding: 12px;
		line-height: 1.45;
	}

	.all-rules {
		margin-top: 14px;
	}

	.all-rules summary {
		cursor: pointer;
		user-select: none;
		font-weight: 900;
		color: #334155;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		padding: 10px 2px;
	}

	.count {
		font-family: var(--mono);
		font-size: 0.75rem;
		color: var(--muted);
		background: rgba(148, 163, 184, 0.14);
		border: 1px solid rgba(148, 163, 184, 0.22);
		border-radius: 999px;
		padding: 2px 8px;
	}

	.rules-list {
		display: grid;
		gap: 10px;
		padding-top: 10px;
	}

	.error-banner {
		background: rgba(254, 226, 226, 0.9);
		border: 1px solid rgba(239, 68, 68, 0.25);
		color: #991b1b;
		padding: 12px 14px;
		border-radius: 12px;
		text-align: center;
	}
</style>
