<svelte:options runes={false} />

<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
	import type { SiteSeo } from '$lib/seo/site';
	import { applyPwaUpdate, dismissPwaUpdateBanner, pwaUpdateBanner } from '$lib/pwa-update';

	export let data: { siteSeo: SiteSeo };

	$: seo = {
		...data.siteSeo,
		...(($page.data?.seo as Partial<SiteSeo> | undefined) ?? {})
	} satisfies SiteSeo;
</script>

<svelte:head>
	<title>{seo.title}</title>
	<meta name="description" content={seo.description} />
	<link rel="canonical" href={seo.canonical} />

	<meta name="robots" content="index,follow,max-snippet:-1,max-image-preview:large,max-video-preview:-1" />
	<meta name="googlebot" content="index,follow,max-snippet:-1,max-image-preview:large,max-video-preview:-1" />

	<meta property="og:type" content={seo.ogType} />
	<meta property="og:site_name" content={seo.siteName} />
	<meta property="og:locale" content={seo.locale} />
	<meta property="og:url" content={seo.canonical} />
	<meta property="og:title" content={seo.title} />
	<meta property="og:description" content={seo.description} />
	<meta property="og:image" content={seo.ogImage} />

	<meta name="twitter:card" content={seo.twitterCard} />
	<meta name="twitter:title" content={seo.title} />
	<meta name="twitter:description" content={seo.description} />

	<meta name="application-name" content={seo.siteName} />
	<meta name="referrer" content="strict-origin-when-cross-origin" />
	<meta name="format-detection" content="telephone=no" />
	<meta name="color-scheme" content="light dark" />

	<link rel="icon" href="/favicon.svg" />
	<link rel="icon" type="image/png" sizes="32x32" href="/icons/icon-32.png" />
	<link rel="icon" type="image/png" sizes="192x192" href="/icons/icon-192.png" />
	<link rel="apple-touch-icon" href="/icons/apple-touch-icon.png" />
	<link rel="manifest" href="/site.webmanifest" />

	<script type="application/ld+json">{JSON.stringify(seo.jsonLd)}</script>
</svelte:head>

<main class="main">
	<slot />
</main>

{#if $pwaUpdateBanner.visible}
	<div class="pwa-update" role="status" aria-live="polite">
		<div class="pwa-update-text">检测到新版本，可点击刷新更新。</div>
		<div class="pwa-update-actions">
			<button class="btn btn-primary" type="button" on:click={applyPwaUpdate} disabled={$pwaUpdateBanner.applying}>
				刷新
			</button>
			<button class="btn btn-ghost" type="button" on:click={dismissPwaUpdateBanner} disabled={$pwaUpdateBanner.applying}>
				稍后
			</button>
		</div>
	</div>
{/if}

<style>
	.pwa-update {
		position: fixed;
		left: 12px;
		right: 12px;
		bottom: 12px;
		z-index: 9998;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		padding: 10px 12px;
		border-radius: var(--radius-lg);
		background: color-mix(in srgb, var(--panel) 92%, #000);
		border: 1px solid var(--border);
		box-shadow: var(--shadow-soft);
	}

	.pwa-update-text {
		color: var(--text);
		font-size: 0.95rem;
	}

	.pwa-update-actions {
		display: flex;
		gap: 8px;
		flex: 0 0 auto;
	}
</style>
