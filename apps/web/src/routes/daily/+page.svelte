<svelte:options runes={false} />

<script lang="ts">
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import A11yToggle from '$lib/components/A11yToggle.svelte';
	import { browser } from '$app/environment';
	import { onMount } from 'svelte';
	import { listProgressEntries, type ProgressEntry } from '$lib/progress.js';

	let todayYmd = '';
	let viewYear = 1970;
	let viewMonth = 1; // 1..12
	let dailyEntries: ProgressEntry[] = [];

	const weekLabels = ['一', '二', '三', '四', '五', '六', '日'];

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

	function pad2(v: number): string {
		return String(v).padStart(2, '0');
	}

	function formatDuration(ms?: number): string {
		if (typeof ms !== 'number' || !Number.isFinite(ms)) return '--';
		const t = Math.max(0, Math.floor(ms / 1000));
		const s = t % 60;
		const m = Math.floor(t / 60) % 60;
		const h = Math.floor(t / 3600);
		const p = (x: number) => String(x).padStart(2, '0');
		if (h > 0) return `${h}:${p(m)}:${p(s)}`;
		return `${m}:${p(s)}`;
	}

	function daysInMonthUtc(year: number, month1: number): number {
		return new Date(Date.UTC(year, month1, 0)).getUTCDate();
	}

	function ymdOf(year: number, month1: number, day: number): string {
		return `${year}-${pad2(month1)}-${pad2(day)}`;
	}

	function buildDailyMap(entries: ProgressEntry[]) {
		const m = new Map<string, ProgressEntry>();
		for (const e of entries) {
			if (e.kind !== 'daily') continue;
			if (typeof e.dateYmd !== 'string' || !e.dateYmd) continue;
			m.set(e.dateYmd, e);
		}
		return m;
	}

	function buildCalendarCells(year: number, month1: number) {
		const firstDowSun0 = new Date(Date.UTC(year, month1 - 1, 1)).getUTCDay(); // 0=Sun
		const offsetMon0 = (firstDowSun0 + 6) % 7; // 0=Mon
		const days = daysInMonthUtc(year, month1);
		const cells: ({ ymd: string; day: number } | null)[] = [];
		for (let i = 0; i < offsetMon0; i++) cells.push(null);
		for (let d = 1; d <= days; d++) cells.push({ ymd: ymdOf(year, month1, d), day: d });
		while (cells.length % 7 !== 0) cells.push(null);
		return cells;
	}

	function prevMonth() {
		if (viewMonth === 1) {
			viewYear -= 1;
			viewMonth = 12;
		} else {
			viewMonth -= 1;
		}
	}

	function nextMonth() {
		if (viewMonth === 12) {
			viewYear += 1;
			viewMonth = 1;
		} else {
			viewMonth += 1;
		}
	}

	function gotoDaily(ymd: string) {
		if (!browser) return;
		const url = new URL(window.location.origin + '/');
		url.searchParams.set('date', ymd);
		window.location.href = url.toString();
	}

	$: dailyMap = buildDailyMap(dailyEntries);
	$: calendarCells = buildCalendarCells(viewYear, viewMonth);
	$: monthSolved = calendarCells
		.filter((c): c is { ymd: string; day: number } => !!c)
		.filter((c) => !!dailyMap.get(c.ymd)?.solvedAt).length;
	$: monthTotal = calendarCells.filter(Boolean).length;
	$: totalSolved = dailyEntries.filter((e) => e.kind === 'daily' && !!e.solvedAt).length;
	$: todayStreak = (() => {
		if (!todayYmd) return 0;
		const solved = new Set(
			dailyEntries.filter((e) => e.kind === 'daily' && !!e.solvedAt && typeof e.dateYmd === 'string').map((e) => e.dateYmd!)
		);
		let streak = 0;
		let cursor = todayYmd;
		for (let i = 0; i < 3660; i++) {
			if (!solved.has(cursor)) break;
			streak += 1;
			const dt = new Date(cursor + 'T00:00:00Z');
			dt.setUTCDate(dt.getUTCDate() - 1);
			cursor = `${dt.getUTCFullYear()}-${pad2(dt.getUTCMonth() + 1)}-${pad2(dt.getUTCDate())}`;
		}
		return streak;
	})();

	onMount(() => {
		todayYmd = shanghaiDateYmd();
		viewYear = Number(todayYmd.slice(0, 4)) || 1970;
		viewMonth = Number(todayYmd.slice(5, 7)) || 1;
		dailyEntries = listProgressEntries();
	});
</script>

<div class="page">
	<header class="header">
		<div class="header-left">
			<a class="btn back" href="/">返回</a>
			<div class="title-area">
				<h1 class="title">每日题日历</h1>
				<p class="subtitle">回顾往期每日题，继续挑战</p>
			</div>
		</div>
		<div class="header-right">
			<a class="btn" href="/stats">统计</a>
			<a class="btn" href="/editor">编辑器</a>
			<A11yToggle />
			<ThemeToggle />
		</div>
	</header>

	<div class="layout">
		<section class="card cal">
			<div class="cal-head">
				<div class="month">
					<button class="btn" type="button" on:click={prevMonth} aria-label="上个月">‹</button>
					<div class="month-label">{viewYear}-{pad2(viewMonth)}</div>
					<button class="btn" type="button" on:click={nextMonth} aria-label="下个月">›</button>
				</div>
				<div class="summary">
					<span class="chip" title="本月通关数">{monthSolved}/{monthTotal}</span>
					<span class="chip" title="累计通关数">{totalSolved}</span>
					<span class="chip" title="连续通关（截至今日）">{todayStreak} 天</span>
				</div>
			</div>

			<div class="week">
				{#each weekLabels as w}
					<div class="week-cell">{w}</div>
				{/each}
			</div>

			<div class="grid" role="grid" aria-label="每日题日历">
				{#each calendarCells as cell, idx (idx)}
					{#if cell}
						{@const entry = dailyMap.get(cell.ymd) ?? null}
						{@const solved = !!entry?.solvedAt}
						<button
							type="button"
							class="day {cell.ymd === todayYmd ? 'today' : ''} {solved ? 'solved' : ''}"
							on:click={() => gotoDaily(cell.ymd)}
							title={solved
								? `已通关｜提示 ${entry?.hintCount ?? 0} 次｜用时 ${formatDuration(entry?.timeMs)}`
								: '未通关 / 未记录'}
						>
							<div class="day-num">{cell.day}</div>
							{#if solved}
								<div class="day-meta">
									<span class="dot"></span>
									<span class="mini">{formatDuration(entry?.timeMs)}</span>
								</div>
							{:else if entry}
								<div class="day-meta">
									<span class="mini">进行中</span>
								</div>
							{/if}
						</button>
					{:else}
						<div class="day empty" aria-hidden="true"></div>
					{/if}
				{/each}
			</div>
		</section>

		<aside class="side">
			<div class="card panel">
				<div class="panel-title">说明</div>
				<div class="hint">
					<div>点击日期即可打开该日每日题（会跳转到主页并携带 `date=YYYY-MM-DD`）。</div>
					<div>通关记录仅保存在浏览器本地。</div>
				</div>
			</div>

			<div class="card panel">
				<div class="panel-title">最近通关</div>
				{#if dailyEntries.filter((e) => e.kind === 'daily' && !!e.solvedAt).length === 0}
					<div class="empty">暂无通关记录</div>
				{:else}
					<div class="list">
						{#each [...dailyEntries]
							.filter((e) => e.kind === 'daily' && !!e.solvedAt && typeof e.dateYmd === 'string')
							.sort((a, b) => String(b.dateYmd ?? '').localeCompare(String(a.dateYmd ?? '')))
							.slice(0, 10) as e (e.key)}
							<button class="row" type="button" on:click={() => gotoDaily(e.dateYmd!)}>
								<span class="mono">{e.dateYmd}</span>
								<span class="muted">提示 {e.hintCount ?? 0}</span>
								<span class="muted">用时 {formatDuration(e.timeMs)}</span>
							</button>
						{/each}
					</div>
				{/if}
			</div>
		</aside>
	</div>
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

	.header-right {
		display: flex;
		align-items: center;
		gap: 10px;
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

	.cal {
		padding: 14px 14px;
	}

	.cal-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		margin-bottom: 10px;
		flex-wrap: wrap;
	}

	.month {
		display: inline-flex;
		align-items: center;
		gap: 10px;
	}

	.month-label {
		font-weight: 900;
		font-size: 1.05rem;
		letter-spacing: -0.02em;
	}

	.summary {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		justify-content: flex-end;
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

	.week {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		gap: 8px;
		margin: 10px 0 8px;
	}

	.week-cell {
		color: var(--muted);
		font-size: 0.82rem;
		font-weight: 800;
		text-align: center;
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		gap: 8px;
	}

	.day {
		position: relative;
		border: 1px solid var(--border);
		background: var(--panel);
		border-radius: var(--radius-sm);
		min-height: 74px;
		padding: 8px 8px;
		cursor: pointer;
		display: grid;
		align-content: start;
		gap: 8px;
		box-shadow: var(--inset-highlight);
	}

	.day:hover:not(.empty) {
		border-color: var(--border-2);
		background: var(--panel-hover);
	}

	.day.empty {
		border: 1px dashed var(--border);
		background: transparent;
		cursor: default;
		box-shadow: none;
	}

	.day-num {
		font-weight: 900;
	}

	.day-meta {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--muted);
		font-size: 0.78rem;
	}

	.dot {
		width: 8px;
		height: 8px;
		border-radius: 999px;
		background: var(--success);
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--success) 22%, transparent);
	}

	.day.solved {
		border-color: color-mix(in srgb, var(--success) 55%, var(--border));
	}

	.day.today {
		box-shadow:
			0 0 0 3px color-mix(in srgb, var(--c-blue) 22%, transparent),
			var(--inset-highlight);
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

	.hint {
		color: var(--muted);
		font-size: 0.9rem;
		display: grid;
		gap: 6px;
	}

	.empty {
		color: var(--muted);
		font-size: 0.9rem;
		padding: 4px 2px;
	}

	.list {
		display: grid;
		gap: 8px;
	}

	.row {
		border: 1px solid var(--border);
		background: var(--panel);
		border-radius: var(--radius-sm);
		padding: 10px 10px;
		display: flex;
		gap: 10px;
		align-items: center;
		justify-content: space-between;
		cursor: pointer;
		box-shadow: var(--inset-highlight);
	}

	.row:hover {
		border-color: var(--border-2);
		background: var(--panel-hover);
	}

	.mono {
		font-family: var(--mono);
		font-weight: 800;
	}

	.muted {
		color: var(--muted);
		font-size: 0.85rem;
		white-space: nowrap;
	}
</style>
