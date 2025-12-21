// 统计与成就（完全基于本地存档汇总，无后端）
//
// 设计：
// - 统计“清空”不删除存档，仅写入 resetAt 截断时间窗口（避免误删进度）
// - 汇总逻辑尽量纯函数化，便于单测

export const STATS_RESET_AT_KEY = 'kairem.stats.resetAt';

/**
 * @param {any} entry
 * @param {string|null} resetAtIso
 */
export function shouldIncludeEntry(entry, resetAtIso) {
	if (!resetAtIso) return true;
	if (!entry || typeof entry !== 'object') return false;
	const updatedAt = typeof entry.updatedAt === 'string' ? entry.updatedAt : '';
	const solvedAt = typeof entry.solvedAt === 'string' ? entry.solvedAt : '';
	return Boolean((updatedAt && updatedAt >= resetAtIso) || (solvedAt && solvedAt >= resetAtIso));
}

/**
 * @param {string} ymd
 * @param {number} deltaDays
 */
export function ymdAddDays(ymd, deltaDays) {
	const s = String(ymd ?? '').trim();
	if (!/^\d{4}-\d{2}-\d{2}$/.test(s)) throw new Error(`bad ymd: ${s}`);
	const dt = new Date(`${s}T00:00:00Z`);
	if (Number.isNaN(dt.getTime())) throw new Error(`bad ymd: ${s}`);
	dt.setUTCDate(dt.getUTCDate() + deltaDays);
	const y = dt.getUTCFullYear();
	const m = String(dt.getUTCMonth() + 1).padStart(2, '0');
	const d = String(dt.getUTCDate()).padStart(2, '0');
	return `${y}-${m}-${d}`;
}

/**
 * @param {Set<string>} solvedYmd
 * @param {string} todayYmd
 * @param {number} maxDays
 */
export function computeDailyStreak(solvedYmd, todayYmd, maxDays = 3660) {
	let streak = 0;
	let cursor = todayYmd;
	for (let i = 0; i < maxDays; i++) {
		if (!solvedYmd.has(cursor)) break;
		streak += 1;
		cursor = ymdAddDays(cursor, -1);
	}
	return streak;
}

/**
 * @param {any[]} entries
 * @param {string|null} resetAtIso
 */
export function summarizeByKind(entries, resetAtIso) {
	/** @type {{ kind: string, total: number, solved: number }} */
	const out = {
		daily: { kind: 'daily', total: 0, solved: 0 },
		seed: { kind: 'seed', total: 0, solved: 0 },
		custom: { kind: 'custom', total: 0, solved: 0 }
	};

	for (const e of entries ?? []) {
		if (!shouldIncludeEntry(e, resetAtIso)) continue;
		const k = e.kind;
		if (k !== 'daily' && k !== 'seed' && k !== 'custom') continue;
		out[k].total += 1;
		if (typeof e.solvedAt === 'string' && e.solvedAt) out[k].solved += 1;
	}

	return out;
}

/**
 * 最近 N 天（按日期）序列：只聚合 daily（有 dateYmd）。
 *
 * @param {string} todayYmd
 * @param {number} days
 * @param {any[]} entries
 * @param {string|null} resetAtIso
 */
export function buildDailyTrend(todayYmd, days, entries, resetAtIso) {
	const dailyMap = new Map();
	for (const e of entries ?? []) {
		if (e?.kind !== 'daily') continue;
		if (!shouldIncludeEntry(e, resetAtIso)) continue;
		if (typeof e.dateYmd !== 'string' || !e.dateYmd) continue;
		dailyMap.set(e.dateYmd, e);
	}

	const out = [];
	for (let i = days - 1; i >= 0; i--) {
		const ymd = ymdAddDays(todayYmd, -i);
		const e = dailyMap.get(ymd) ?? null;
		const solved = !!(e && typeof e.solvedAt === 'string' && e.solvedAt);
		out.push({
			ymd,
			played: !!e,
			solved,
			timeMs: typeof e?.timeMs === 'number' && Number.isFinite(e.timeMs) ? Math.max(0, e.timeMs) : null,
			hintCount: typeof e?.hintCount === 'number' && Number.isFinite(e.hintCount) ? Math.max(0, e.hintCount) : null,
			moveCount: typeof e?.moveCount === 'number' && Number.isFinite(e.moveCount) ? Math.max(0, e.moveCount) : null
		});
	}
	return out;
}

/**
 * @param {any[]} entries
 * @param {string|null} resetAtIso
 */
export function bestDaily(entries, resetAtIso) {
	let fastest = null;
	let leastHints = null;

	for (const e of entries ?? []) {
		if (e?.kind !== 'daily') continue;
		if (!shouldIncludeEntry(e, resetAtIso)) continue;
		if (!(typeof e.solvedAt === 'string' && e.solvedAt)) continue;

		const timeMs = typeof e.timeMs === 'number' && Number.isFinite(e.timeMs) ? Math.max(0, e.timeMs) : null;
		const hintCount = typeof e.hintCount === 'number' && Number.isFinite(e.hintCount) ? Math.max(0, e.hintCount) : null;

		if (timeMs !== null) {
			if (!fastest || timeMs < fastest.timeMs) fastest = { dateYmd: e.dateYmd ?? '', timeMs };
		}
		if (hintCount !== null) {
			if (!leastHints || hintCount < leastHints.hintCount) leastHints = { dateYmd: e.dateYmd ?? '', hintCount };
		}
	}

	return { fastest, leastHints };
}
