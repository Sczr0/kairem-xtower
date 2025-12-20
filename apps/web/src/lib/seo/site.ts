import { env } from '$env/dynamic/public';
import { normalizeSiteUrl, toAbsoluteUrl } from './url.js';

export type SiteSeo = {
	lang: string;
	locale: string;
	siteName: string;
	title: string;
	description: string;
	canonical: string;
	ogType: 'website';
	ogImage: string;
	twitterCard: 'summary';
	jsonLd: unknown;
};

const FALLBACK_SITE_URL = 'https://kairem.xtower.site';
const FALLBACK_SITE_NAME = '聪明Bingo';
const FALLBACK_DESCRIPTION =
	'一个轻量的逻辑益智游戏：根据规则勾选格子，达成 Bingo。支持每日题与自定义种子分享。';

function getSiteUrl(): string {
	const normalized = normalizeSiteUrl(env.PUBLIC_SITE_URL);
	return normalized || FALLBACK_SITE_URL;
}

function getSiteName(): string {
	const name = (env.PUBLIC_SITE_NAME ?? '').trim();
	return name || FALLBACK_SITE_NAME;
}

export function buildSiteSeo(pathname: string): SiteSeo {
	const siteUrl = getSiteUrl();
	const siteName = getSiteName();
	const canonical = toAbsoluteUrl(siteUrl, pathname);

	return {
		lang: 'zh-CN',
		locale: 'zh_CN',
		siteName,
		title: siteName,
		description: FALLBACK_DESCRIPTION,
		canonical,
		ogType: 'website',
		ogImage: toAbsoluteUrl(siteUrl, '/og.svg'),
		twitterCard: 'summary',
		jsonLd: buildJsonLd({ siteUrl, siteName, canonical })
	};
}

function buildJsonLd(input: { siteUrl: string; siteName: string; canonical: string }): unknown {
	return [
		{
			'@context': 'https://schema.org',
			'@type': 'WebSite',
			name: input.siteName,
			url: normalizeSiteUrl(input.siteUrl),
			inLanguage: 'zh-CN'
		},
		{
			'@context': 'https://schema.org',
			'@type': 'WebApplication',
			name: input.siteName,
			url: input.canonical,
			applicationCategory: 'GameApplication',
			operatingSystem: 'All'
		}
	];
}
