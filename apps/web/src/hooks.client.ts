import { dev } from '$app/environment';
import { setupPwaUpdate } from '$lib/pwa-update';

// PWA：仅在生产环境注册 Service Worker（避免开发时缓存干扰调试）
export const init = async () => {
	if (dev) return;
	if (typeof window === 'undefined') return;
	if (!('serviceWorker' in navigator)) return;

	try {
		const registration = await navigator.serviceWorker.register('/service-worker.js');
		setupPwaUpdate(registration);
	} catch {
		// 忽略：注册失败不会影响主功能
	}
};

export const handleError = () => {
	// 默认不拦截错误；保持 SvelteKit 默认行为
};
