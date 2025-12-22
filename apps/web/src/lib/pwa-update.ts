// PWA 更新提示（Service Worker 更新时提示“新版本可用”）
//
// 设计目标：
// - 不引入后端；纯前端实现
// - 尽量遵循 Service Worker 的标准更新流程：新 SW 安装后进入 waiting，用户点击“刷新”后再激活并 reload
// - UI 仅做轻量提示，不强依赖具体页面

import { writable } from 'svelte/store';

export type PwaUpdateBannerState = {
	visible: boolean;
	applying: boolean;
	lastDetectedAt: number | null;
};

export const pwaUpdateBanner = writable<PwaUpdateBannerState>({
	visible: false,
	applying: false,
	lastDetectedAt: null
});

let currentRegistration: ServiceWorkerRegistration | null = null;
let controllerChangeListenerBound = false;

function markUpdateAvailable() {
	pwaUpdateBanner.update((s) => ({
		...s,
		visible: true,
		lastDetectedAt: Date.now()
	}));
}

function bindControllerChangeReloadOnce() {
	if (controllerChangeListenerBound) return;
	controllerChangeListenerBound = true;

	navigator.serviceWorker.addEventListener('controllerchange', () => {
		let shouldReload = false;
		pwaUpdateBanner.update((s) => {
			shouldReload = s.applying;
			return s;
		});
		if (!shouldReload) return;
		// 新 SW 已接管，刷新页面加载新构建产物
		window.location.reload();
	});
}

export function setupPwaUpdate(registration: ServiceWorkerRegistration) {
	currentRegistration = registration;

	if (!('serviceWorker' in navigator)) return;
	bindControllerChangeReloadOnce();

	// 若已存在 waiting 且当前页面已有 controller，说明这是一次更新（非首次安装）
	if (registration.waiting && navigator.serviceWorker.controller) {
		markUpdateAvailable();
	}

	registration.addEventListener('updatefound', () => {
		const newWorker = registration.installing;
		if (!newWorker) return;

		newWorker.addEventListener('statechange', () => {
			// installed 且已有 controller：表示新版本已就绪（waiting）
			if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
				markUpdateAvailable();
			}
		});
	});

	// 定时检查更新：避免用户长时间打开页面看不到更新提示
	// 说明：浏览器也会自行检查，但加一层“保底”更稳。
	setInterval(() => {
		currentRegistration?.update().catch(() => {
			// 忽略：离线/受限环境可能更新失败
		});
	}, 60 * 60 * 1000);
}

export function dismissPwaUpdateBanner() {
	pwaUpdateBanner.update((s) => ({ ...s, visible: false }));
}

export async function applyPwaUpdate() {
	const reg = currentRegistration;
	if (!reg) return;
	if (!reg.waiting) {
		// 没有 waiting 时直接尝试检查更新；即使失败也不阻塞用户
		try {
			await reg.update();
		} catch {
			// ignore
		}
	}

	if (!reg.waiting) return;

	pwaUpdateBanner.update((s) => ({ ...s, applying: true }));

	// 触发 waiting SW 立刻激活（配合 SW 里的 message 监听）
	reg.waiting.postMessage({ type: 'SKIP_WAITING' });
}

