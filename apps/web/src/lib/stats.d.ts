export const STATS_RESET_AT_KEY: string;

export function shouldIncludeEntry(entry: any, resetAtIso: string | null): boolean;
export function ymdAddDays(ymd: string, deltaDays: number): string;
export function computeDailyStreak(solvedYmd: Set<string>, todayYmd: string, maxDays?: number): number;
export function summarizeByKind(entries: any[], resetAtIso: string | null): {
	daily: { kind: 'daily'; total: number; solved: number };
	seed: { kind: 'seed'; total: number; solved: number };
	custom: { kind: 'custom'; total: number; solved: number };
};
export function buildDailyTrend(
	todayYmd: string,
	days: number,
	entries: any[],
	resetAtIso: string | null
): { ymd: string; played: boolean; solved: boolean; timeMs: number | null; hintCount: number | null; moveCount: number | null }[];
export function bestDaily(
	entries: any[],
	resetAtIso: string | null
): { fastest: { dateYmd: string; timeMs: number } | null; leastHints: { dateYmd: string; hintCount: number } | null };

