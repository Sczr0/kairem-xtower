import { env } from '$env/dynamic/public';
import { renderSitemapXml } from '$lib/seo/url.js';
import type { RequestHandler } from './$types';

export const prerender = true;

export const GET: RequestHandler = () => {
	const siteUrl = (env.PUBLIC_SITE_URL ?? '').trim() || 'https://kairem.xtower.site';
	const now = new Date().toISOString();
	const body = renderSitemapXml({
		origin: siteUrl,
		urls: [
			{ loc: '/', lastmod: now, changefreq: 'daily', priority: '1.0' },
			{ loc: '/daily', lastmod: now, changefreq: 'daily', priority: '0.7' },
			{ loc: '/editor', lastmod: now, changefreq: 'weekly', priority: '0.6' }
		]
	});

	return new Response(body, {
		headers: {
			'content-type': 'application/xml; charset=utf-8',
			'cache-control': 'public, max-age=3600'
		}
	});
};
