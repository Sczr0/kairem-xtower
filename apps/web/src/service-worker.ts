/// <reference lib="webworker" />

import { build, files, prerendered, version } from '$service-worker';

// PWA 离线缓存（SvelteKit 官方推荐写法的轻量变体）
//
// 目标：
// - 预缓存构建产物与静态文件（含 wasm、manifest、favicon）
// - 导航请求 network-first，离线时回退到缓存的 `/`
// - 静态资源 cache-first，提升离线可用性与加载速度

const CACHE_NAME = `kairem-pwa-${version}`;
const ASSETS = Array.from(new Set([...build, ...files, ...prerendered, '/']));

self.addEventListener('install', (event) => {
	event.waitUntil(
		(async () => {
			const cache = await caches.open(CACHE_NAME);
			await cache.addAll(ASSETS);
			await self.skipWaiting();
		})()
	);
});

self.addEventListener('activate', (event) => {
	event.waitUntil(
		(async () => {
			const keys = await caches.keys();
			await Promise.all(keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k)));
			await self.clients.claim();
		})()
	);
});

function isSameOrigin(request: Request) {
	try {
		const url = new URL(request.url);
		return url.origin === self.location.origin;
	} catch {
		return false;
	}
}

self.addEventListener('fetch', (event) => {
	const request = event.request;
	if (request.method !== 'GET') return;
	if (!isSameOrigin(request)) return;

	const url = new URL(request.url);
	const isAsset =
		url.pathname.startsWith('/_app/') ||
		url.pathname === '/favicon.svg' ||
		url.pathname === '/site.webmanifest' ||
		url.pathname === '/service-worker.js' ||
		files.includes(url.pathname as any) ||
		build.includes(url.pathname as any);

	// 导航：network-first，离线回退到首页（应用壳）
	if (request.mode === 'navigate') {
		event.respondWith(
			(async () => {
				try {
					const response = await fetch(request);
					const cache = await caches.open(CACHE_NAME);
					cache.put(request, response.clone());
					return response;
				} catch {
					const cached = await caches.match(request);
					if (cached) return cached;
					return (await caches.match('/')) || Response.error();
				}
			})()
		);
		return;
	}

	// 静态资源：cache-first
	if (isAsset) {
		event.respondWith(
			(async () => {
				const cached = await caches.match(request);
				if (cached) return cached;
				const response = await fetch(request);
				const cache = await caches.open(CACHE_NAME);
				cache.put(request, response.clone());
				return response;
			})()
		);
		return;
	}

	// 其他 GET：network-first（兜底到 cache）
	event.respondWith(
		(async () => {
			try {
				const response = await fetch(request);
				const cache = await caches.open(CACHE_NAME);
				cache.put(request, response.clone());
				return response;
			} catch {
				return (await caches.match(request)) || Response.error();
			}
		})()
	);
});
