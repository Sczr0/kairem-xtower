import test from 'node:test';
import assert from 'node:assert/strict';

import {
	normalizeSiteUrl,
	renderRobotsTxt,
	renderSitemapXml,
	stripQueryAndHash,
	toAbsoluteUrl
} from '../src/lib/seo/url.js';

test('normalizeSiteUrl: trims and strips path', () => {
	assert.equal(normalizeSiteUrl(' https://example.com/abc/ '), 'https://example.com');
	assert.equal(normalizeSiteUrl(''), '');
});

test('toAbsoluteUrl: builds absolute from pathname', () => {
	assert.equal(toAbsoluteUrl('https://example.com', '/a'), 'https://example.com/a');
	assert.equal(toAbsoluteUrl('https://example.com/', 'a'), 'https://example.com/a');
});

test('stripQueryAndHash: removes both', () => {
	assert.equal(stripQueryAndHash('https://example.com/a?b=1#c'), 'https://example.com/a');
});

test('renderRobotsTxt: includes sitemap and allow', () => {
	const txt = renderRobotsTxt({ origin: 'https://example.com', allowAll: true, disallow: [] });
	assert.match(txt, /User-agent: \*/);
	assert.match(txt, /Allow: \//);
	assert.match(txt, /Sitemap: https:\/\/example\.com\/sitemap\.xml/);
});

test('renderSitemapXml: emits urlset with loc', () => {
	const xml = renderSitemapXml({
		origin: 'https://example.com',
		urls: [{ loc: '/', lastmod: '2025-01-01T00:00:00.000Z', changefreq: 'daily', priority: '1.0' }]
	});
	assert.match(xml, /<urlset/);
	assert.match(xml, /<loc>https:\/\/example\.com\/<\/loc>/);
	assert.match(xml, /<lastmod>2025-01-01T00:00:00\.000Z<\/lastmod>/);
});
