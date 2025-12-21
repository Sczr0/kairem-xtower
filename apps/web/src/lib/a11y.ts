import { writable } from 'svelte/store';

// 无障碍/色盲相关设置（仅前端 UI）
//
// 设计目标：
// - 默认关闭色盲模式（避免增加视觉噪声）
// - 设置落地到 documentElement.dataset，便于 CSS 控制
// - 同步写入 localStorage，刷新后保持

const STORAGE_KEY_COLORBLIND = 'a11y.colorBlind';

export type ColorBlindMode = 'on' | 'off';

function readColorBlindFromDom(): boolean {
	return document.documentElement?.dataset?.colorblind === 'on';
}

function readColorBlindFromStorage(): boolean | null {
	try {
		const raw = localStorage.getItem(STORAGE_KEY_COLORBLIND);
		if (raw === 'on') return true;
		if (raw === 'off') return false;
		return null;
	} catch {
		return null;
	}
}

function applyColorBlind(next: boolean) {
	document.documentElement.dataset.colorblind = next ? 'on' : 'off';
	try {
		localStorage.setItem(STORAGE_KEY_COLORBLIND, next ? 'on' : 'off');
	} catch {
		// 忽略：无痕/隐私模式可能禁用 localStorage
	}
}

export const colorBlindEnabled = writable(false);

export function initColorBlind() {
	const fromStorage = readColorBlindFromStorage();
	if (fromStorage !== null) {
		applyColorBlind(fromStorage);
		colorBlindEnabled.set(fromStorage);
		return;
	}

	const fromDom = readColorBlindFromDom();
	colorBlindEnabled.set(fromDom);
}

export function setColorBlindEnabled(next: boolean) {
	applyColorBlind(next);
	colorBlindEnabled.set(next);
}

export function toggleColorBlind() {
	let current = readColorBlindFromDom();
	const next = !current;
	setColorBlindEnabled(next);
}

