export const prerender = true;

import type { PageLoad } from './$types';

export const load: PageLoad = () => {
	return {
		seo: {
			title: '统计与成就｜聪明Bingo',
			description:
				'基于本地存档的轻量统计与成就：最近 7/30 天趋势、连续通关、最快用时、最少提示等。无需后端。'
		}
	};
};

