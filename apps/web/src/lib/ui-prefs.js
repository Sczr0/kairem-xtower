// UI 偏好设置（本地存储，无后端）
//
// 设计目标：
// - 用于“仅影响 UI 的开关/折叠状态”等偏好；
// - 仅依赖 localStorage，且在无痕/隐私模式下失败时静默降级；
// - Node 环境可通过注入 storage 进行单元测试。

export const STORAGE_KEY_PROGRESS_LIST_OPEN = 'ui.progressListOpen';

/**
 * @typedef {{ getItem?: (key: string) => string | null, setItem?: (key: string, value: string) => void }} StorageLike
 */

/**
 * @returns {StorageLike | null}
 */
function getGlobalStorage() {
	// 通过 globalThis 兼容浏览器/Node；Node 下通常没有 localStorage
	const ls = /** @type {unknown} */ (globalThis)?.localStorage;
	if (!ls) return null;
	return /** @type {StorageLike} */ (ls);
}

/**
 * @param {StorageLike | null | undefined} storage
 * @param {string} key
 * @param {boolean} defaultValue
 */
export function readBooleanFlag(storage, key, defaultValue) {
	if (!storage || typeof storage.getItem !== 'function') return defaultValue;
	try {
		const raw = storage.getItem(key);
		if (raw === '1') return true;
		if (raw === '0') return false;
		return defaultValue;
	} catch {
		return defaultValue;
	}
}

/**
 * @param {StorageLike | null | undefined} storage
 * @param {string} key
 * @param {boolean} value
 */
export function writeBooleanFlag(storage, key, value) {
	if (!storage || typeof storage.setItem !== 'function') return;
	try {
		storage.setItem(key, value ? '1' : '0');
	} catch {
		// 忽略：无痕/隐私模式可能禁用 localStorage
	}
}

/**
 * 读取“存档列表是否展开”的 UI 偏好。
 * @param {StorageLike | null | undefined} [storage]
 */
export function readProgressListOpen(storage = getGlobalStorage()) {
	return readBooleanFlag(storage, STORAGE_KEY_PROGRESS_LIST_OPEN, true);
}

/**
 * 写入“存档列表是否展开”的 UI 偏好。
 * @param {boolean} value
 * @param {StorageLike | null | undefined} [storage]
 */
export function writeProgressListOpen(value, storage = getGlobalStorage()) {
	writeBooleanFlag(storage, STORAGE_KEY_PROGRESS_LIST_OPEN, value);
}
