import { env } from '$env/dynamic/public';
import { renderRobotsTxt } from '$lib/seo/url.js';
import type { RequestHandler } from './$types';

export const prerender = true;

export const GET: RequestHandler = () => {
	const siteUrl = (env.PUBLIC_SITE_URL ?? '').trim() || 'https://kairem.xtower.site';
	const body = renderRobotsTxt({
		origin: siteUrl,
		allowAll: true,
		disallow: []
	});

	return new Response(body, {
		headers: {
			'content-type': 'text/plain; charset=utf-8',
			'cache-control': 'public, max-age=3600'
		}
	});
};
