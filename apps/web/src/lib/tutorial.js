// 新手引导（首次进入弹出，可关闭，可从页面再次打开）
//
// 设计约束：
// - 不引入后端；仅依赖 localStorage
// - “不再自动弹出”通过写入 dismissedAt 实现（可随时清除恢复）

export const TUTORIAL_VERSION = 1;
export const TUTORIAL_DISMISSED_AT_KEY = `kairem.tutorial.v${TUTORIAL_VERSION}.dismissedAt`;

/**
 * @param {any} raw
 * @returns {string|null}
 */
export function normalizeDismissedAt(raw) {
	if (typeof raw !== 'string') return null;
	const s = raw.trim();
	if (!s) return null;
	// 仅做轻量校验：ISO 时间字符串开头应是 YYYY-MM-DDT
	if (!/^\d{4}-\d{2}-\d{2}T/.test(s)) return null;
	return s;
}

/**
 * @param {string|null} dismissedAtIso
 */
export function shouldAutoShowTutorial(dismissedAtIso) {
	return !normalizeDismissedAt(dismissedAtIso);
}

function canUseLocalStorage() {
	try {
		return typeof localStorage !== 'undefined';
	} catch {
		return false;
	}
}

/**
 * @returns {string|null}
 */
export function readTutorialDismissedAt() {
	if (!canUseLocalStorage()) return null;
	try {
		return normalizeDismissedAt(localStorage.getItem(TUTORIAL_DISMISSED_AT_KEY));
	} catch {
		return null;
	}
}

/**
 * @param {boolean} dismissed
 */
export function setTutorialDismissed(dismissed) {
	if (!canUseLocalStorage()) return;
	try {
		if (!dismissed) {
			localStorage.removeItem(TUTORIAL_DISMISSED_AT_KEY);
			return;
		}
		localStorage.setItem(TUTORIAL_DISMISSED_AT_KEY, new Date().toISOString());
	} catch {
		// 忽略：无痕/隐私模式可能禁用 localStorage
	}
}

