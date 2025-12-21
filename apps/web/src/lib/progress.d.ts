export const PROGRESS_STORE_VERSION: 1;
export const PROGRESS_STORAGE_KEY: string;
export const PROGRESS_MAX_ENTRIES: number;
export const HISTORY_LIMIT: number;

export type PuzzleKind = 'daily' | 'seed' | 'custom';

export type PuzzleKeyInfo = { dateYmd?: string; seed?: string; levelCode?: string };

export function makePuzzleKey(kind: PuzzleKind, info: PuzzleKeyInfo): string;
export function normalizeMaskU32(mask: number): number;

export type HistoryState = { undo: number[]; redo: number[]; present: number };
export function normalizeHistory(history: { undo?: number[]; redo?: number[]; present: number }, limit?: number): HistoryState;
export function createHistory(initialMask: number): HistoryState;
export function historyPush(history: HistoryState, nextMask: number): HistoryState;
export function historyUndo(history: HistoryState): HistoryState;
export function historyRedo(history: HistoryState): HistoryState;

export type ProgressEntry = {
	key: string;
	kind: PuzzleKind;
	dateYmd?: string;
	seed?: string;
	levelCode?: string;
	checkedMask: number;
	undo: number[];
	redo: number[];
	moveCount?: number;
	hintCount?: number;
	solvedAt?: string;
	createdAt?: string;
	updatedAt?: string;
};

export function loadProgressEntry(key: string): ProgressEntry | null;
export function listProgressEntries(): ProgressEntry[];
export function upsertProgressEntry(entry: ProgressEntry): void;
export function deleteProgressEntry(key: string): void;
export function clearAllProgress(): void;

