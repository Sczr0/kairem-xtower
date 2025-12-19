<svelte:options runes={false} />

<script lang="ts">
	import Matrix from '$lib/components/Matrix.svelte';
	import rules from '$lib/rules.json';
	import { Color, type ColorId, colorToCss } from '$lib/colors';
	import { loadEngine, type Engine, type ValidateResult } from '$lib/wasm/load';
	import { onMount } from 'svelte';

	let engine: Engine | null = null;
	let engineError = '';

	let dateYmd = '';
	let seed: bigint | null = null;

	let grid2d: number[][] = [];
	let grid: ColorId[] = Array.from({ length: 25 }, () => Color.White);
	let checkedMask = 0;
	let validate: ValidateResult | null = null;
	let hoveredRuleId: string | null = null;

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

	function toggle(i: number) {
		if (grid[i] === Color.Black) return;
		checkedMask = (checkedMask ^ (1 << i)) >>> 0;
		refreshValidate();
	}

	function handleHover(index: number | null) {
		if (index === null) {
			hoveredRuleId = null;
			return;
		}
		const color = grid[index];
		hoveredRuleId = colorRuleMap[color] ?? null;
	}

	async function newDailyPuzzle() {
		if (!engine) return;
		dateYmd = shanghaiDateYmd();
		seed = engine.date_to_seed_ymd(dateYmd);
		grid2d = engine.generate_puzzle(seed);
		grid = flattenGrid2d(grid2d);
		checkedMask = blackMaskFromGrid(grid);
		refreshValidate();
	}

	onMount(async () => {
		try {
			engine = await loadEngine();
			await newDailyPuzzle();
		} catch (e) {
			engineError = String(e);
		}
	});
</script>

<div class="page-container">
	<header class="header">
		<h1 class="title">Kairem</h1>
		<p class="subtitle">无需服务器交互的轻量级逻辑游戏</p>
	</header>

	{#if engineError}
		<div class="error-banner">{engineError}</div>
	{:else}
		<div class="game-area">
			<div class="matrix-container">
				<div class="daily-badge">
					<span class="label">DAILY PUZZLE</span>
					<span class="value">{dateYmd || '—'}</span>
				</div>

				<Matrix
					grid={grid}
					checkedMask={checkedMask}
					cellOk={validate?.cell_ok ?? Array(25).fill(true)}
					cellMessages={validate?.cell_messages ?? Array(25).fill(undefined)}
					onToggle={toggle}
					onHover={handleHover}
				/>

				<div class="debug-info">
					<span class="pill">mask: {checkedMask >>> 0}</span>
					{#if validate}
						<span class="pill {validate.is_bingo ? 'success' : ''}">bingo: {validate.is_bingo}</span>
						<span class="pill {validate.is_valid ? 'success' : ''}">valid: {validate.is_valid}</span>
					{/if}
				</div>
			</div>

			<div class="rules-hud">
				<div class="hud-header">
					<h2>MISSION OBJECTIVES</h2>
				</div>

				<div class="rules-list">
					{#each rules.rules as r}
						{@const colorId = ruleColorMap[r.id] ?? Color.White}
						{@const isHighlighted = hoveredRuleId === r.id}
						<div class="rule-card {isHighlighted ? 'highlighted' : ''}">
							<div class="rule-icon" style="color: {r.id === 'bingo' ? '#f59e0b' : colorToCss(colorId)}">
								{r.id.toUpperCase().slice(0, 1)}
							</div>
							<div class="rule-content">
								<div class="rule-header">
									<span class="rule-name">{r.name}</span>
									<span class="badge {r.appliesWhen}">{r.appliesWhen}</span>
								</div>
								<div class="rule-desc">{r.description}</div>
							</div>
						</div>
					{/each}
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.page-container {
		max-width: 1000px;
		margin: 0 auto;
		padding: 40px 20px;
		font-family: 'Inter', system-ui, sans-serif;
	}

	.header {
		margin-bottom: 40px;
		text-align: center;
	}

	.title {
		font-size: 2.5rem;
		font-weight: 800;
		letter-spacing: -0.05em;
		margin-bottom: 0.5rem;
		background: linear-gradient(135deg, #0f172a 0%, #334155 100%);
		background-clip: text;
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
	}

	.subtitle {
		color: #64748b;
		font-size: 1.1rem;
	}

	.game-area {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 40px;
	}

	.rules-hud {
		width: 100%;
		max-width: 600px;
	}

	.matrix-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 20px;
	}

	.daily-badge {
		display: flex;
		align-items: center;
		gap: 8px;
		background: #f1f5f9;
		padding: 6px 12px;
		border-radius: 999px;
		font-size: 0.875rem;
		font-weight: 600;
		color: #475569;
		margin-bottom: 10px;
	}

	.daily-badge .label {
		color: #94a3b8;
		font-size: 0.75rem;
		letter-spacing: 0.05em;
	}

	.debug-info {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
		justify-content: center;
		opacity: 0.7;
	}

	.pill {
		font-family: monospace;
		font-size: 0.75rem;
		background: #f1f5f9;
		padding: 2px 6px;
		border-radius: 4px;
	}

	.pill.success {
		background: #dcfce7;
		color: #166534;
	}

	.rules-hud {
		background: #ffffff;
		border: 1px solid #e2e8f0;
		border-radius: 16px;
		padding: 20px;
		box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
	}

	.hud-header h2 {
		font-size: 0.75rem;
		font-weight: 800;
		letter-spacing: 0.1em;
		color: #94a3b8;
		margin-bottom: 16px;
		text-transform: uppercase;
	}

	.rules-list {
		display: grid;
		gap: 8px;
	}

	.rule-card {
		display: flex;
		gap: 12px;
		padding: 10px;
		border-radius: 8px;
		border: 1px solid transparent;
		background: #f8fafc;
		transition: all 0.2s;
	}

	.rule-card.highlighted {
		background: #ffffff;
		border-color: #94a3b8;
		box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -1px rgb(0 0 0 / 0.06);
		transform: scale(1.02);
	}

	.rule-icon {
		font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
		font-size: 0.9rem;
		display: grid;
		place-items: center;
		width: 24px;
		height: 24px;
		border-radius: 8px;
		background: rgba(148, 163, 184, 0.12);
		flex-shrink: 0;
	}

	.rule-content {
		flex: 1;
		min-width: 0;
	}

	.rule-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 4px;
	}

	.rule-name {
		font-weight: 700;
		font-size: 0.9rem;
		color: #334155;
	}

	.badge {
		font-size: 0.65rem;
		padding: 2px 6px;
		border-radius: 999px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.02em;
	}

	.badge.always {
		background: #e2e8f0;
		color: #64748b;
	}

	.badge.checkedOnly {
		background: #e0f2fe;
		color: #0284c7;
	}

	.badge.goal {
		background: #fef3c7;
		color: #d97706;
	}

	.rule-desc {
		font-size: 0.8rem;
		color: #64748b;
		line-height: 1.4;
	}

	.error-banner {
		background: #fee2e2;
		color: #991b1b;
		padding: 1rem;
		border-radius: 0.5rem;
		margin-bottom: 2rem;
		text-align: center;
	}
</style>
