export const PROGRESS_STORE_VERSION: 1;
export const PROGRESS_STORAGE_KEY: string;
export const PROGRESS_MAX_ENTRIES: number;
export const HISTORY_LIMIT: number;

export type PuzzleKind = 'daily' | 'seed' | 'custom';

export type PuzzleKeyInfo = { dateYmd?: string; seed?: string; levelCode?: string };

export function makePuzzleKey(kind: PuzzleKind, info: PuzzleKeyInfo): string;
export function normalizeMask(mask: bigint | number | string): bigint;

export type HistoryState = { undo: bigint[]; redo: bigint[]; present: bigint };
export function normalizeHistory(history: { undo?: (bigint | number | string)[]; redo?: (bigint | number | string)[]; present: bigint | number | string }, limit?: number): HistoryState;
export function createHistory(initialMask: bigint | number | string): HistoryState;
export function historyPush(history: HistoryState, nextMask: bigint | number | string): HistoryState;
export function historyUndo(history: HistoryState): HistoryState;
export function historyRedo(history: HistoryState): HistoryState;

export type ProgressEntry = {
	key: string;
	kind: PuzzleKind;
	dateYmd?: string;
	seed?: string;
	levelCode?: string;
	checkedMask: bigint | number | string;
	marks?: number[];
	undo: (bigint | number | string)[];
	redo: (bigint | number | string)[];
	moveCount?: number;
	hintCount?: number;
	timeMs?: number;
	solvedAt?: string;
	createdAt?: string;
	updatedAt?: string;
};

export function loadProgressEntry(key: string): ProgressEntry | null;
export function listProgressEntries(): ProgressEntry[];
export function upsertProgressEntry(entry: ProgressEntry): void;
export function deleteProgressEntry(key: string): void;
export function clearAllProgress(): void;
