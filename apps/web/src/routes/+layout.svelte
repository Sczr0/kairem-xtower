<svelte:options runes={false} />

<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
	import type { SiteSeo } from '$lib/seo/site';

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
	<link rel="apple-touch-icon" href="/favicon.svg" />
	<link rel="manifest" href="/site.webmanifest" />

	<script type="application/ld+json">{JSON.stringify(seo.jsonLd)}</script>
</svelte:head>

<main class="main">
	<slot />
</main>
