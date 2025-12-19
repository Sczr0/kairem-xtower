<svelte:options runes={false} />

<script lang="ts">
	import Matrix from '$lib/components/Matrix.svelte';
	import RuleCard, { type Rule as UiRule } from '$lib/components/RuleCard.svelte';
	import rules from '$lib/rules.json';
	import { Color, type ColorId, colorToCss } from '$lib/colors';
	import { loadEngine, type Engine, type ValidateResult } from '$lib/wasm/load';
	import { browser, dev } from '$app/environment';
	import { onMount } from 'svelte';
    import { fade, slide } from 'svelte/transition';

	// --- 逻辑部分保持不变 ---
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

<div class="page-container">
	{#if engineError}
		<div class="error-banner">引擎加载失败：{engineError}</div>
	{:else}
		<header class="header">
            <div class="logo-area">
			    <h1 class="logo">Kairem</h1>
			    <p class="subtitle">轻量逻辑游戏</p>
            </div>
            <!-- 移动端可以将 Action 放这里，或者保持在下面 -->
		</header>

		<div class="main-layout">
			<!-- 左侧/上方：游戏主区域 -->
			<main class="game-area">
                
                <!-- 工具栏：整合信息与操作 -->
				<div class="toolbar">
					<div class="game-info">
						<span class="info-label">{puzzleKind === 'daily' ? '今日题目' : '随机种子'}</span>
						<span class="info-value font-mono">
                            {puzzleKind === 'daily' ? (dateYmd || '—') : (seed ? shortSeed(seed) : '—')}
                        </span>
					</div>

					<div class="game-actions">
						<button class="btn btn-primary" on:click={sharePuzzle} disabled={!seed} title="分享题目">
							分享
						</button>
						<button class="btn" on:click={newRandomPuzzle} disabled={!engine}>
							随机
						</button>
						{#if puzzleKind !== 'daily'}
							<button class="btn btn-ghost" on:click={() => newDailyPuzzle({ updateUrl: true })} disabled={!engine}>
								回到今日
							</button>
						{/if}
					</div>
				</div>

				{#if shareToast}
					<div class="toast" transition:slide={{ axis: 'y' }}>{shareToast}</div>
				{/if}

				{#if shareManualVisible && shareUrlForManualCopy}
					<div class="share-manual" transition:slide>
						<input class="input-copy" readonly value={shareUrlForManualCopy} on:focus={selectAll} on:click={selectAll} />
					</div>
				{/if}

				{#if urlSeedError}
					<div class="hint-banner">{urlSeedError}</div>
				{/if}

                <!-- 棋盘容器 -->
				<div class="matrix-wrapper">
					<Matrix
						grid={grid}
						checkedMask={checkedMask}
						cellOk={validate?.cell_ok ?? Array(25).fill(true)}
						cellMessages={validate?.cell_messages ?? Array(25).fill(undefined)}
						onToggle={toggle}
						onHover={handleHover}
					/>
				</div>

                <!-- 状态指示条 -->
				{#if validate}
					<div class="status-bar">
                        <div class="status-item {validate.is_valid ? 'status-success' : 'status-warn'}">
                            <span class="status-dot"></span>
                            <span>约束：{validate.is_valid ? '满足' : '未满足'}</span>
                        </div>
                        <div class="status-item {validate.is_bingo ? 'status-success' : 'status-neutral'}">
                            <span class="status-dot"></span>
                            <span>目标：{validate.is_bingo ? '达成' : '进行中'}</span>
                        </div>
					</div>
				{/if}

				{#if dev}
					<details class="debug-panel">
						<summary>调试信息</summary>
						<div class="debug-content">
							<code>seed: {seed?.toString() ?? '—'}</code>
							<code>mask: {checkedMask >>> 0}</code>
						</div>
					</details>
				{/if}
			</main>

			<!-- 右侧：规则与帮助 -->
			<aside class="sidebar">
                <!-- 1. 帮助移到这里，并设为折叠，节省空间 -->
                <div class="sidebar-card help-section">
                    <details>
                        <summary class="help-summary">怎么玩？</summary>
                        <ul class="help-list">
                            <li><strong>点击非黑格：</strong>切换勾选状态。</li>
                            <li><strong>黑格：</strong>已锁定，必须勾选。</li>
                            <li><strong>红框/叹号：</strong>违反规则，悬停查看详情。</li>
                        </ul>
                    </details>
                </div>

                <!-- 2. 规则面板 -->
				<div class="sidebar-card rules-panel">
					<div class="panel-header">
						<h2 class="panel-title">规则详情</h2>
                        {#if !hoveredRule}
						    <p class="panel-hint">悬停或点击格子定位规则</p>
                        {/if}
					</div>

                    <!-- 动态高亮区域 -->
					<div class="active-rule-section">
						{#if hoveredRule}
                            <div class="section-label">当前关注</div>
							<div transition:slide={{ duration: 200 }}>
                                <RuleCard rule={hoveredRule} color={ruleColorCss(hoveredRule.id)} highlighted />
                            </div>
						{:else}
							<div class="empty-placeholder">
                                <span>👆 移动鼠标查看规则</span>
                            </div>
						{/if}
					</div>

					{#if goalRule}
						<div class="static-rule-section">
							<div class="section-label">通关目标</div>
							<RuleCard rule={goalRule} color={ruleColorCss(goalRule.id)} />
						</div>
					{/if}

					<details class="all-rules-details">
						<summary>
							全部规则 <span class="badge-count">{allRules.length}</span>
						</summary>
						<div class="rules-grid">
							{#each allRules as r}
								<RuleCard rule={r} color={ruleColorCss(r.id)} highlighted={hoveredRuleId === r.id} />
							{/each}
						</div>
					</details>
				</div>
			</aside>
		</div>
	{/if}
</div>

<style>
	.page-container {
		max-width: 1180px;
		margin: 0 auto;
	}

    /* 字体工具 */
    .font-mono { font-family: var(--mono); }

	/* Header */
	.header {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
		gap: 16px;
		margin-bottom: 18px;
	}

	.logo-area {
		display: grid;
		gap: 4px;
	}

	.logo {
		font-size: 2rem;
		font-weight: 900;
		letter-spacing: -0.06em;
		margin: 0;
		background: linear-gradient(110deg, #38bdf8 0%, #a855f7 45%, #fb7185 100%);
		-webkit-background-clip: text;
		background-clip: text;
		color: transparent;
	}

	.subtitle {
		margin: 0;
		color: var(--muted);
		font-size: 0.95rem;
		font-weight: 550;
	}

	/* Main Layout */
	.main-layout {
		display: grid;
		gap: 18px;
		align-items: start;
	}

	@media (min-width: 900px) {
		.main-layout {
			grid-template-columns: minmax(0, 1fr) 360px;
		}
        .sidebar {
            position: sticky;
            top: 24px;
        }
	}

	/* Panels */
	.game-area,
	.sidebar {
		background: var(--panel-2);
		border: 1px solid var(--border);
		border-radius: var(--radius-xl);
		box-shadow: var(--shadow-soft), var(--inset-highlight);
	}

	.game-area {
		padding: 16px 16px 18px;
	}

	.sidebar {
		padding: 16px;
	}

    /* --- Game Area Styles --- */

    /* Toolbar */
    .toolbar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 12px;
        padding-bottom: 14px;
        margin-bottom: 14px;
        border-bottom: 1px solid var(--border);
        flex-wrap: wrap;
    }

    .game-info {
        display: flex;
        align-items: baseline;
        gap: 10px;
        flex-wrap: wrap;
    }
    
    .info-label {
        font-size: 0.72rem;
        text-transform: uppercase;
        color: var(--muted-2);
        font-weight: 800;
        letter-spacing: 0.08em;
    }

    .info-value {
        font-weight: 650;
        font-size: 1rem;
        color: var(--text);
    }

    .game-actions {
        display: flex;
        gap: 10px;
    }

    /* Buttons */
    .btn {
        appearance: none;
        border: 1px solid rgba(148, 163, 184, 0.22);
        background: rgba(248, 250, 252, 0.06);
        color: rgba(248, 250, 252, 0.92);
        padding: 8px 14px;
        border-radius: var(--radius-sm);
        font-size: 0.9rem;
        font-weight: 650;
        cursor: pointer;
        box-shadow: var(--inset-highlight);
        transition:
            background-color 120ms ease,
            border-color 120ms ease,
            transform 80ms ease;
    }

    .btn:hover:not(:disabled) {
        background: rgba(248, 250, 252, 0.1);
        border-color: rgba(148, 163, 184, 0.32);
    }

    .btn:active:not(:disabled) {
        transform: translateY(1px);
    }

    .btn:disabled { opacity: 0.55; cursor: not-allowed; }

    .btn-primary {
        background: linear-gradient(110deg, #38bdf8 0%, #a855f7 60%, #fb7185 120%);
        color: #070a14;
        border-color: rgba(248, 250, 252, 0.08);
    }
    .btn-primary:hover:not(:disabled) {
        border-color: rgba(248, 250, 252, 0.22);
    }

    .btn-ghost {
        background: transparent;
        border-color: transparent;
        color: var(--muted);
        box-shadow: none;
    }
    .btn-ghost:hover:not(:disabled) {
        background: rgba(248, 250, 252, 0.06);
        border-color: transparent;
    }

    /* Matrix Container */
    .matrix-wrapper {
        display: flex;
        justify-content: center;
        background: linear-gradient(180deg, rgba(248, 250, 252, 0.05), rgba(248, 250, 252, 0.02));
        padding: 18px;
        border-radius: var(--radius-xl);
        border: 1px solid rgba(148, 163, 184, 0.24);
        box-shadow: var(--inset-highlight), var(--inset-shadow);
    }
    
    @media (min-width: 900px) {
        .matrix-wrapper { justify-content: center; min-height: 480px; align-items: center; }
    }

    /* Status Bar */
    .status-bar {
        display: flex;
        gap: 10px;
        margin-top: 14px;
        justify-content: center;
        flex-wrap: wrap;
    }
    @media (min-width: 900px) { .status-bar { justify-content: flex-start; } }

    .status-item {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 12px;
        border-radius: 99px;
        font-size: 0.9rem;
        font-weight: 650;
        background: rgba(248, 250, 252, 0.06);
        border: 1px solid rgba(148, 163, 184, 0.22);
        box-shadow: var(--inset-highlight);
    }

    .status-dot {
        width: 9px;
        height: 9px;
        border-radius: 50%;
        background: rgba(148, 163, 184, 0.7);
    }

    .status-success { color: rgba(236, 253, 245, 0.95); border-color: rgba(16, 185, 129, 0.22); background: rgba(16, 185, 129, 0.12); }
    .status-success .status-dot { background: rgba(16, 185, 129, 0.95); }

    .status-warn { color: rgba(254, 242, 242, 0.95); border-color: rgba(251, 113, 133, 0.22); background: rgba(251, 113, 133, 0.12); }
    .status-warn .status-dot { background: rgba(251, 113, 133, 0.95); }

    .status-neutral { color: rgba(248, 250, 252, 0.92); }

    /* --- Sidebar Styles --- */

    .sidebar-card {
        background: transparent;
        border: 0;
        box-shadow: none;
        overflow: visible;
        margin-bottom: 16px;
    }

    .help-section {
        font-size: 0.92rem;
    }
    .help-summary {
        padding: 10px 12px;
        font-weight: 800;
        cursor: pointer;
        user-select: none;
        background: rgba(248, 250, 252, 0.06);
        border: 1px solid rgba(148, 163, 184, 0.22);
        border-radius: var(--radius-md);
        color: rgba(248, 250, 252, 0.92);
        box-shadow: var(--inset-highlight);
    }
    .help-list {
        margin: 10px 0 0;
        padding: 10px 12px 0 26px;
        color: rgba(248, 250, 252, 0.82);
        line-height: 1.55;
    }

    .rules-panel {
        padding: 0;
    }

    .panel-header {
        margin: 18px 0 14px;
        padding-top: 14px;
        border-top: 1px solid var(--border);
    }

    .panel-title {
        font-size: 1.08rem;
        font-weight: 850;
        margin: 0;
        color: rgba(248, 250, 252, 0.92);
    }

    .panel-hint {
        font-size: 0.85rem;
        color: var(--muted);
        margin: 6px 0 0 0;
    }

    .section-label {
        font-size: 0.72rem;
        text-transform: uppercase;
        color: var(--muted-2);
        font-weight: 900;
        letter-spacing: 0.08em;
        margin-bottom: 8px;
    }

    .active-rule-section {
        min-height: 80px; /* 避免高度跳动 */
        margin-bottom: 16px;
    }

    .empty-placeholder {
        display: grid;
        place-items: center;
        height: 60px;
        background: rgba(248, 250, 252, 0.04);
        border-radius: var(--radius-md);
        border: 1px dashed rgba(148, 163, 184, 0.28);
        color: var(--muted);
        font-size: 0.9rem;
    }

    .static-rule-section { margin-bottom: 14px; }

    .all-rules-details summary {
        cursor: pointer;
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-size: 0.95rem;
        font-weight: 750;
        color: rgba(248, 250, 252, 0.9);
        padding: 10px 0;
        border-top: 1px solid var(--border);
    }

    .badge-count {
        background: rgba(148, 163, 184, 0.12);
        border: 1px solid rgba(148, 163, 184, 0.22);
        color: rgba(248, 250, 252, 0.82);
        font-size: 0.8rem;
        padding: 2px 8px;
        border-radius: 99px;
    }
    
    .rules-grid {
        display: grid;
        gap: 10px;
        padding-top: 10px;
    }

    /* Toast & Utils */
    .toast {
        background: rgba(2, 6, 23, 0.68);
        border: 1px solid var(--border);
        color: rgba(248, 250, 252, 0.95);
        padding: 10px 12px;
        border-radius: var(--radius-sm);
        text-align: center;
        font-size: 0.92rem;
        margin-bottom: 12px;
        box-shadow: var(--shadow-chip), var(--inset-highlight);
    }

    .share-manual {
        margin-bottom: 12px;
    }

    .input-copy {
        width: 100%;
        padding: 10px 10px;
        border: 1px solid rgba(148, 163, 184, 0.25);
        border-radius: var(--radius-sm);
        background: rgba(2, 6, 23, 0.35);
        color: rgba(248, 250, 252, 0.92);
        font-family: var(--mono);
    }

    .hint-banner {
        background: rgba(251, 113, 133, 0.12);
        border: 1px solid rgba(251, 113, 133, 0.25);
        color: rgba(248, 250, 252, 0.92);
        padding: 10px 12px;
        border-radius: var(--radius-sm);
        margin-bottom: 12px;
    }

    .error-banner {
        background: rgba(251, 113, 133, 0.12);
        border: 1px solid rgba(251, 113, 133, 0.25);
        color: rgba(248, 250, 252, 0.92);
        padding: 12px 14px;
        border-radius: var(--radius-md);
        text-align: center;
        margin-bottom: 16px;
    }

    .debug-panel { margin-top: 16px; opacity: 0.6; font-size: 0.85rem; }

    .debug-content {
        display: grid;
        gap: 6px;
        padding-top: 10px;
    }
</style>
