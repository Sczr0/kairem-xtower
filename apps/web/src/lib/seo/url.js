/**
 * SEO 相关的 URL 工具（纯函数，便于单元测试）。
 * 注意：该文件刻意不依赖 SvelteKit 的 $env / $app 等模块。
 */

/**
 * @param {string} siteUrl
 * @returns {string}
 */
export function normalizeSiteUrl(siteUrl) {
	const raw = (siteUrl ?? '').trim();
	if (!raw) return '';
	try {
		const u = new URL(raw);
		return `${u.protocol}//${u.host}`;
	} catch {
		return '';
	}
}

/**
 * @param {string} siteUrl - 站点根 URL（如 https://example.com）
 * @param {string} pathname - 以 / 开头的路径
 * @returns {string}
 */
export function toAbsoluteUrl(siteUrl, pathname) {
	const base = normalizeSiteUrl(siteUrl);
	if (!base) return pathname || '/';
	const path = (pathname ?? '').startsWith('/') ? pathname : `/${pathname ?? ''}`;
	return new URL(path, base).toString();
}

/**
 * @param {string} href
 * @returns {string}
 */
export function stripQueryAndHash(href) {
	try {
		const u = new URL(href);
		u.search = '';
		u.hash = '';
		return u.toString();
	} catch {
		const [noHash] = String(href ?? '').split('#');
		const [noQuery] = noHash.split('?');
		return noQuery || '';
	}
}

/**
 * @param {{ origin: string; urls: Array<{ loc: string; lastmod?: string; changefreq?: string; priority?: string }>; }} input
 * @returns {string}
 */
export function renderSitemapXml(input) {
	const origin = normalizeSiteUrl(input?.origin ?? '');
	const urls = Array.isArray(input?.urls) ? input.urls : [];
	const safeUrls = urls
		.map((u) => ({
			loc: u?.loc ? toAbsoluteUrl(origin, u.loc) : '',
			lastmod: u?.lastmod,
			changefreq: u?.changefreq,
			priority: u?.priority
		}))
		.filter((u) => Boolean(u.loc));

	const body = safeUrls
		.map((u) => {
			const parts = [
				`<loc>${escapeXml(u.loc)}</loc>`,
				u.lastmod ? `<lastmod>${escapeXml(u.lastmod)}</lastmod>` : '',
				u.changefreq ? `<changefreq>${escapeXml(u.changefreq)}</changefreq>` : '',
				u.priority ? `<priority>${escapeXml(u.priority)}</priority>` : ''
			].filter(Boolean);
			return `<url>${parts.join('')}</url>`;
		})
		.join('');

	return (
		`<?xml version="1.0" encoding="UTF-8"?>` +
		`<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">${body}</urlset>`
	);
}

/**
 * @param {{ origin: string; disallow?: string[]; allowAll?: boolean; }} input
 * @returns {string}
 */
export function renderRobotsTxt(input) {
	const origin = normalizeSiteUrl(input?.origin ?? '');
	const disallow = Array.isArray(input?.disallow) ? input.disallow : [];
	const allowAll = input?.allowAll ?? true;

	const lines = ['User-agent: *'];
	if (allowAll) lines.push('Allow: /');
	else lines.push('Disallow: /');

	for (const p of disallow) {
		if (!p) continue;
		lines.push(`Disallow: ${p}`);
	}

	if (origin) {
		lines.push(`Sitemap: ${toAbsoluteUrl(origin, '/sitemap.xml')}`);
	}

	return `${lines.join('\n')}\n`;
}

/**
 * @param {string} s
 * @returns {string}
 */
function escapeXml(s) {
	return String(s)
		.replaceAll('&', '&amp;')
		.replaceAll('<', '&lt;')
		.replaceAll('>', '&gt;')
		.replaceAll('"', '&quot;')
		.replaceAll("'", '&apos;');
}

