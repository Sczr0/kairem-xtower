// 存档/历史 + 撤销/重做（本地存储）
//
// 设计目标：
// - 仅依赖 localStorage，不引入额外库；
// - 数据结构可版本化升级；
// - 逻辑尽量纯函数，便于单测与复用。

export const PROGRESS_STORE_VERSION = 1;
export const PROGRESS_STORAGE_KEY = 'kairem.progress.v1';
export const PROGRESS_MAX_ENTRIES = 30;
export const HISTORY_LIMIT = 200;

/**
 * @param {'daily'|'seed'|'custom'} kind
 * @param {{ dateYmd?: string, seed?: string, levelCode?: string }} info
 */
export function makePuzzleKey(kind, info) {
	if (kind === 'daily') {
		if (!info?.dateYmd) throw new Error('daily 需要 dateYmd');
		return `daily:${info.dateYmd}`;
	}
	if (kind === 'seed') {
		if (!info?.seed) throw new Error('seed 需要 seed');
		return `seed:${info.seed}`;
	}
	if (kind === 'custom') {
		if (!info?.levelCode) throw new Error('custom 需要 levelCode');
		return `level:${info.levelCode}`;
	}
	throw new Error(`未知 kind：${String(kind)}`);
}

/**
 * @param {number} mask
 */
export function normalizeMaskU32(mask) {
	// 确保落在 u32 范围内（并保持 JS number）
	return (mask >>> 0) >>> 0;
}

/**
 * @param {{ undo?: number[], redo?: number[], present: number }} history
 * @param {number} limit
 */
export function normalizeHistory(history, limit = HISTORY_LIMIT) {
	const undo = Array.isArray(history.undo) ? history.undo.map(normalizeMaskU32) : [];
	const redo = Array.isArray(history.redo) ? history.redo.map(normalizeMaskU32) : [];
	const present = normalizeMaskU32(history.present);
	return {
		undo: undo.slice(Math.max(0, undo.length - limit)),
		redo: redo.slice(Math.max(0, redo.length - limit)),
		present
	};
}

/**
 * @param {number} initialMask
 */
export function createHistory(initialMask) {
	return normalizeHistory({ undo: [], redo: [], present: initialMask });
}

/**
 * 推入新状态：将 current 进 undo，清空 redo。
 * @param {{ undo: number[], redo: number[], present: number }} history
 * @param {number} nextMask
 */
export function historyPush(history, nextMask) {
	const h = normalizeHistory(history);
	const next = normalizeMaskU32(nextMask);
	if (h.present === next) return h;
	const undo = [...h.undo, h.present];
	return normalizeHistory({ undo, redo: [], present: next });
}

/**
 * 撤销：present -> redo，undo.pop() -> present
 * @param {{ undo: number[], redo: number[], present: number }} history
 */
export function historyUndo(history) {
	const h = normalizeHistory(history);
	if (h.undo.length === 0) return h;
	const prev = h.undo[h.undo.length - 1];
	const undo = h.undo.slice(0, -1);
	const redo = [...h.redo, h.present];
	return normalizeHistory({ undo, redo, present: prev });
}

/**
 * 重做：present -> undo，redo.pop() -> present
 * @param {{ undo: number[], redo: number[], present: number }} history
 */
export function historyRedo(history) {
	const h = normalizeHistory(history);
	if (h.redo.length === 0) return h;
	const next = h.redo[h.redo.length - 1];
	const redo = h.redo.slice(0, -1);
	const undo = [...h.undo, h.present];
	return normalizeHistory({ undo, redo, present: next });
}

function canUseLocalStorage() {
	try {
		return typeof localStorage !== 'undefined';
	} catch {
		return false;
	}
}

/**
 * @returns {{ version: number, entries: Record<string, any> }}
 */
function readStore() {
	if (!canUseLocalStorage()) return { version: PROGRESS_STORE_VERSION, entries: {} };
	try {
		const raw = localStorage.getItem(PROGRESS_STORAGE_KEY);
		if (!raw) return { version: PROGRESS_STORE_VERSION, entries: {} };
		const parsed = JSON.parse(raw);
		if (!parsed || typeof parsed !== 'object') return { version: PROGRESS_STORE_VERSION, entries: {} };
		if (parsed.version !== PROGRESS_STORE_VERSION) return { version: PROGRESS_STORE_VERSION, entries: {} };
		if (!parsed.entries || typeof parsed.entries !== 'object') return { version: PROGRESS_STORE_VERSION, entries: {} };
		return parsed;
	} catch {
		return { version: PROGRESS_STORE_VERSION, entries: {} };
	}
}

/**
 * @param {{ version: number, entries: Record<string, any> }} store
 */
function writeStore(store) {
	if (!canUseLocalStorage()) return;
	localStorage.setItem(PROGRESS_STORAGE_KEY, JSON.stringify(store));
}

/**
 * @param {string} key
 */
export function loadProgressEntry(key) {
	const store = readStore();
	return store.entries[key] ?? null;
}

export function listProgressEntries() {
	const store = readStore();
	const all = Object.values(store.entries ?? {});
	all.sort((a, b) => String(b.updatedAt ?? '').localeCompare(String(a.updatedAt ?? '')));
	return all;
}

/**
 * @param {any} entry
 */
export function upsertProgressEntry(entry) {
	if (!entry || typeof entry !== 'object') return;
	if (typeof entry.key !== 'string' || !entry.key) return;

	const store = readStore();
	const nowIso = new Date().toISOString();

	const prev = store.entries[entry.key] ?? null;
	const createdAt = typeof prev?.createdAt === 'string' ? prev.createdAt : nowIso;
	const updatedAt = nowIso;

	const normalizedHistory = normalizeHistory({
		undo: entry.undo ?? [],
		redo: entry.redo ?? [],
		present: entry.checkedMask ?? 0
	});

	store.entries[entry.key] = {
		...prev,
		...entry,
		createdAt,
		updatedAt,
		checkedMask: normalizedHistory.present,
		undo: normalizedHistory.undo,
		redo: normalizedHistory.redo
	};

	// 裁剪历史条目数量：按 updatedAt 排序保留最近 N 条
	const keys = Object.keys(store.entries);
	keys.sort((a, b) => String(store.entries[b]?.updatedAt ?? '').localeCompare(String(store.entries[a]?.updatedAt ?? '')));
	for (let i = PROGRESS_MAX_ENTRIES; i < keys.length; i++) {
		delete store.entries[keys[i]];
	}

	writeStore(store);
}

/**
 * @param {string} key
 */
export function deleteProgressEntry(key) {
	const store = readStore();
	if (store.entries && store.entries[key]) {
		delete store.entries[key];
		writeStore(store);
	}
}

export function clearAllProgress() {
	if (!canUseLocalStorage()) return;
	localStorage.removeItem(PROGRESS_STORAGE_KEY);
}

