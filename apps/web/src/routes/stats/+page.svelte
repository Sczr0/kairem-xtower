<svelte:options runes={false} />

<script lang="ts">
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import A11yToggle from '$lib/components/A11yToggle.svelte';
	import { browser } from '$app/environment';
	import { onMount } from 'svelte';
	import { listProgressEntries, type ProgressEntry } from '$lib/progress.js';
	import {
		STATS_RESET_AT_KEY,
		bestDaily,
		buildDailyTrend,
		computeDailyStreak,
		shouldIncludeEntry,
		summarizeByKind
	} from '$lib/stats.js';

	let entries: ProgressEntry[] = [];
	let resetAtIso: string | null = null;
	let loadedAtIso = '';
	let todayYmd = '';

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

	function formatTimestamp(iso: string): string {
		try {
			return new Date(iso).toLocaleString('zh-CN', { hour12: false });
		} catch {
			return iso;
		}
	}

	function formatDuration(ms: number | null | undefined): string {
		if (typeof ms !== 'number' || !Number.isFinite(ms)) return '--';
		const t = Math.max(0, Math.floor(ms / 1000));
		const s = t % 60;
		const m = Math.floor(t / 60) % 60;
		const h = Math.floor(t / 3600);
		const p = (x: number) => String(x).padStart(2, '0');
		if (h > 0) return `${h}:${p(m)}:${p(s)}`;
		return `${m}:${p(s)}`;
	}

	function reload() {
		entries = listProgressEntries();
		loadedAtIso = new Date().toISOString();
	}

	function readResetAt() {
		if (!browser) return null;
		try {
			const raw = localStorage.getItem(STATS_RESET_AT_KEY);
			if (!raw) return null;
			// 仅接受 ISO 字符串，避免脏数据导致统计异常
			if (!/^\d{4}-\d{2}-\d{2}T/.test(raw)) return null;
			return raw;
		} catch {
			return null;
		}
	}

	function writeResetAt(next: string | null) {
		if (!browser) return;
		try {
			if (!next) localStorage.removeItem(STATS_RESET_AT_KEY);
			else localStorage.setItem(STATS_RESET_AT_KEY, next);
		} catch {
			// 忽略
		}
	}

	function clearStats() {
		const nowIso = new Date().toISOString();
		resetAtIso = nowIso;
		writeResetAt(nowIso);
		reload();
	}

	function restoreStats() {
		resetAtIso = null;
		writeResetAt(null);
		reload();
	}

	$: filtered = entries.filter((e) => shouldIncludeEntry(e, resetAtIso));
	$: byKind = summarizeByKind(entries as any, resetAtIso);
	$: daily7 = todayYmd ? buildDailyTrend(todayYmd, 7, entries as any, resetAtIso) : [];
	$: daily30 = todayYmd ? buildDailyTrend(todayYmd, 30, entries as any, resetAtIso) : [];
	$: dailySolvedSet = new Set(
		filtered.filter((e) => e.kind === 'daily' && !!e.solvedAt && typeof e.dateYmd === 'string').map((e) => e.dateYmd!)
	);
	$: streak = todayYmd ? computeDailyStreak(dailySolvedSet as any, todayYmd) : 0;
	$: best = bestDaily(entries as any, resetAtIso);

	function rowStatus(r: { played: boolean; solved: boolean }) {
		if (r.solved) return '已通关';
		if (r.played) return '进行中';
		return '未开始';
	}

	function avgOfSolved(rows: any[], key: 'timeMs' | 'hintCount' | 'moveCount') {
		const solved = rows.filter((r) => r.solved && typeof r[key] === 'number');
		if (solved.length === 0) return null;
		const sum = solved.reduce((acc, r) => acc + r[key], 0);
		return sum / solved.length;
	}

	$: avg7Time = avgOfSolved(daily7 as any, 'timeMs');
	$: avg7Hints = avgOfSolved(daily7 as any, 'hintCount');
	$: avg7Moves = avgOfSolved(daily7 as any, 'moveCount');
	$: avg30Time = avgOfSolved(daily30 as any, 'timeMs');
	$: avg30Hints = avgOfSolved(daily30 as any, 'hintCount');
	$: avg30Moves = avgOfSolved(daily30 as any, 'moveCount');

	onMount(() => {
		todayYmd = shanghaiDateYmd();
		resetAtIso = readResetAt();
		reload();
	});
</script>

<div class="page">
	<header class="header">
		<div class="header-left">
			<a class="btn back" href="/">返回</a>
			<div class="title-area">
				<h1 class="title">统计与成就</h1>
				<p class="subtitle">完全基于本地存档汇总（无需后端）</p>
			</div>
		</div>
		<div class="header-right">
			<a class="btn" href="/daily">日历</a>
			<a class="btn" href="/editor">编辑器</a>
			<A11yToggle />
			<ThemeToggle />
		</div>
	</header>

	<section class="card panel">
		<div class="panel-head">
			<div class="panel-title">统计口径</div>
			<div class="panel-actions">
				<button class="btn btn-ghost" type="button" on:click={reload}>刷新</button>
				<button class="btn btn-ghost" type="button" on:click={clearStats} title="不删除存档，仅从现在开始重新统计">
					清空统计
				</button>
				<button class="btn btn-ghost" type="button" on:click={restoreStats} disabled={!resetAtIso}>
					恢复全部
				</button>
			</div>
		</div>
		<div class="meta">
			<div class="kv">
				<div class="k">数据更新时间</div>
				<div class="v mono">{loadedAtIso ? formatTimestamp(loadedAtIso) : '--'}</div>
			</div>
			<div class="kv">
				<div class="k">统计窗口</div>
				<div class="v">
					{#if resetAtIso}
						<span class="mono">{formatTimestamp(resetAtIso)}</span>
						<span class="muted">之后更新/完成的记录</span>
					{:else}
						<span class="muted">全部本地记录</span>
					{/if}
				</div>
			</div>
		</div>
	</section>

	<div class="grid">
		<section class="card panel">
			<div class="panel-title">每日题成就</div>
			<div class="chips">
				<span class="chip">连续通关 {streak} 天</span>
				<span class="chip">今日 {todayYmd || '--'}</span>
			</div>
			<div class="meta">
				<div class="kv">
					<div class="k">最快用时（daily）</div>
					<div class="v mono">
						{best.fastest ? `${best.fastest.dateYmd}  ${formatDuration(best.fastest.timeMs)}` : '--'}
					</div>
				</div>
				<div class="kv">
					<div class="k">最少提示（daily）</div>
					<div class="v mono">{best.leastHints ? `${best.leastHints.dateYmd}  ${best.leastHints.hintCount}` : '--'}</div>
				</div>
			</div>
		</section>

		<section class="card panel">
			<div class="panel-title">完成率（本地）</div>
			<div class="meta">
				<div class="kv">
					<div class="k">每日题</div>
					<div class="v mono">{byKind.daily.solved}/{byKind.daily.total}</div>
				</div>
				<div class="kv">
					<div class="k">seed</div>
					<div class="v mono">{byKind.seed.solved}/{byKind.seed.total}</div>
				</div>
				<div class="kv">
					<div class="k">自定义</div>
					<div class="v mono">{byKind.custom.solved}/{byKind.custom.total}</div>
				</div>
			</div>
			<div class="hint muted">说明：仅统计曾打开/保存过的记录；未打开的日期不会计入分母。</div>
		</section>
	</div>

	<section class="card panel">
		<div class="panel-head">
			<div class="panel-title">最近 7 天（通关趋势）</div>
			<div class="chips">
				<span class="chip">均用时 {formatDuration(avg7Time as any)}</span>
				<span class="chip">均提示 {avg7Hints ? Math.round(avg7Hints) : '--'}</span>
				<span class="chip">均步数 {avg7Moves ? Math.round(avg7Moves) : '--'}</span>
			</div>
		</div>

		<div class="table">
			<div class="tr head">
				<div>日期</div>
				<div>状态</div>
				<div>用时</div>
				<div>提示</div>
				<div>步数</div>
			</div>
			{#each daily7 as r (r.ymd)}
				<div class="tr {r.solved ? 'ok' : r.played ? 'mid' : ''}">
					<div class="mono">{r.ymd}</div>
					<div>{rowStatus(r)}</div>
					<div class="mono">{formatDuration(r.timeMs)}</div>
					<div class="mono">{r.hintCount ?? '--'}</div>
					<div class="mono">{r.moveCount ?? '--'}</div>
				</div>
			{/each}
		</div>
	</section>

	<section class="card panel">
		<div class="panel-head">
			<div class="panel-title">最近 30 天（通关趋势）</div>
			<div class="chips">
				<span class="chip">均用时 {formatDuration(avg30Time as any)}</span>
				<span class="chip">均提示 {avg30Hints ? Math.round(avg30Hints) : '--'}</span>
				<span class="chip">均步数 {avg30Moves ? Math.round(avg30Moves) : '--'}</span>
			</div>
		</div>

		<div class="table table-compact">
			<div class="tr head">
				<div>日期</div>
				<div>状态</div>
				<div>用时</div>
				<div>提示</div>
				<div>步数</div>
			</div>
			{#each daily30 as r (r.ymd)}
				<div class="tr {r.solved ? 'ok' : r.played ? 'mid' : ''}">
					<div class="mono">{r.ymd}</div>
					<div>{rowStatus(r)}</div>
					<div class="mono">{formatDuration(r.timeMs)}</div>
					<div class="mono">{r.hintCount ?? '--'}</div>
					<div class="mono">{r.moveCount ?? '--'}</div>
				</div>
			{/each}
		</div>
	</section>
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
		flex-wrap: wrap;
		justify-content: flex-end;
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

	.panel {
		padding: 14px 14px;
	}

	.panel-title {
		font-weight: 900;
		margin-bottom: 10px;
	}

	.panel-head {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 12px;
		flex-wrap: wrap;
		margin-bottom: 10px;
	}

	.panel-actions {
		display: flex;
		gap: 10px;
		flex-wrap: wrap;
	}

	.meta {
		display: grid;
		gap: 8px;
	}

	.kv {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		padding: 8px 0;
		border-top: 1px solid var(--border);
	}

	.kv:first-child {
		border-top: none;
	}

	.k {
		color: var(--muted);
		font-size: 0.9rem;
	}

	.v {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
		justify-content: flex-end;
	}

	.mono {
		font-family: var(--mono);
		font-weight: 800;
	}

	.muted {
		color: var(--muted);
	}

	.hint {
		margin-top: 10px;
		font-size: 0.9rem;
	}

	.grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 16px;
		margin: 16px 0;
	}

	@media (max-width: 980px) {
		.grid {
			grid-template-columns: 1fr;
		}
	}

	.chips {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
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

	.table {
		display: grid;
		gap: 0;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		overflow: hidden;
	}

	.tr {
		display: grid;
		grid-template-columns: 160px 90px 90px 70px 70px;
		gap: 8px;
		padding: 10px 12px;
		background: var(--panel);
		border-top: 1px solid var(--border);
		align-items: center;
	}

	.tr.head {
		background: var(--bg-2);
		font-weight: 900;
		border-top: none;
	}

	.tr.ok {
		background: color-mix(in srgb, var(--success) 8%, var(--panel));
	}

	.tr.mid {
		background: color-mix(in srgb, var(--c-blue) 6%, var(--panel));
	}

	.table-compact .tr {
		padding: 8px 12px;
		font-size: 0.92rem;
	}

	@media (max-width: 720px) {
		.tr {
			grid-template-columns: 1fr 80px 80px 60px 60px;
		}
	}
</style>

