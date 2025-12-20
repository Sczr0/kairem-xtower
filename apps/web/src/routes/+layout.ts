import type { LayoutLoad } from './$types';
import { buildSiteSeo } from '$lib/seo/site';

export const prerender = true;

export const load: LayoutLoad = ({ url }) => {
	return {
		siteSeo: buildSiteSeo(url.pathname)
	};
};

