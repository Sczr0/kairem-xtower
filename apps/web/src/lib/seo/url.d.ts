export function normalizeSiteUrl(siteUrl: string | undefined | null): string;
export function toAbsoluteUrl(siteUrl: string | undefined | null, pathname: string | undefined | null): string;
export function stripQueryAndHash(href: string | undefined | null): string;
export function renderSitemapXml(input: {
	origin: string;
	urls: Array<{ loc: string; lastmod?: string; changefreq?: string; priority?: string }>;
}): string;
export function renderRobotsTxt(input: { origin: string; disallow?: string[]; allowAll?: boolean }): string;

