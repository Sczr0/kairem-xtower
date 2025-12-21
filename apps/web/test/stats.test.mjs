import test from 'node:test';
import assert from 'node:assert/strict';

import { bestDaily, buildDailyTrend, computeDailyStreak, shouldIncludeEntry, summarizeByKind, ymdAddDays } from '../src/lib/stats.js';

test('ymdAddDays: basic', () => {
	assert.equal(ymdAddDays('2025-12-21', 0), '2025-12-21');
	assert.equal(ymdAddDays('2025-12-21', 1), '2025-12-22');
	assert.equal(ymdAddDays('2025-01-01', -1), '2024-12-31');
});

test('shouldIncludeEntry: resetAt filter uses updatedAt/solvedAt', () => {
	const resetAt = '2025-12-22T00:00:00.000Z';
	assert.equal(shouldIncludeEntry({ updatedAt: '2025-12-21T23:59:59.000Z' }, resetAt), false);
	assert.equal(shouldIncludeEntry({ solvedAt: '2025-12-22T00:00:00.000Z' }, resetAt), true);
	assert.equal(shouldIncludeEntry({ updatedAt: '2025-12-22T00:00:00.001Z' }, resetAt), true);
});

test('computeDailyStreak: counts consecutive solved days', () => {
	const solved = new Set(['2025-12-22', '2025-12-21', '2025-12-20']);
	assert.equal(computeDailyStreak(solved, '2025-12-22'), 3);
	assert.equal(computeDailyStreak(solved, '2025-12-21'), 2);
	assert.equal(computeDailyStreak(solved, '2025-12-19'), 0);
});

test('summarizeByKind: counts totals and solved', () => {
	const entries = [
		{ kind: 'daily', solvedAt: 'x' },
		{ kind: 'daily' },
		{ kind: 'seed', solvedAt: 'x' },
		{ kind: 'custom' }
	];
	const s = summarizeByKind(entries, null);
	assert.deepEqual(s.daily, { kind: 'daily', total: 2, solved: 1 });
	assert.deepEqual(s.seed, { kind: 'seed', total: 1, solved: 1 });
	assert.deepEqual(s.custom, { kind: 'custom', total: 1, solved: 0 });
});

test('buildDailyTrend: creates day series with played/solved flags', () => {
	const entries = [
		{ kind: 'daily', dateYmd: '2025-12-21', solvedAt: 'x', timeMs: 1000, hintCount: 2, moveCount: 3 },
		{ kind: 'daily', dateYmd: '2025-12-22' }
	];
	const trend = buildDailyTrend('2025-12-22', 2, entries, null);
	assert.equal(trend.length, 2);
	assert.equal(trend[0].ymd, '2025-12-21');
	assert.equal(trend[0].played, true);
	assert.equal(trend[0].solved, true);
	assert.equal(trend[1].ymd, '2025-12-22');
	assert.equal(trend[1].played, true);
	assert.equal(trend[1].solved, false);
});

test('bestDaily: picks fastest time and least hints', () => {
	const entries = [
		{ kind: 'daily', dateYmd: '2025-12-20', solvedAt: 'x', timeMs: 9000, hintCount: 3 },
		{ kind: 'daily', dateYmd: '2025-12-21', solvedAt: 'x', timeMs: 8000, hintCount: 5 },
		{ kind: 'daily', dateYmd: '2025-12-22', solvedAt: 'x', timeMs: 12000, hintCount: 1 }
	];
	const b = bestDaily(entries, null);
	assert.deepEqual(b.fastest, { dateYmd: '2025-12-21', timeMs: 8000 });
	assert.deepEqual(b.leastHints, { dateYmd: '2025-12-22', hintCount: 1 });
});

