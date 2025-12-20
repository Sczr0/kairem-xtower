export const prerender = true;

import type { PageLoad } from './$types';

export const load: PageLoad = () => {
	return {
		seo: {
			title: '聪明Bingo｜每日逻辑益智游戏',
			description:
				'每日一题的轻量逻辑益智游戏：根据规则勾选格子，达成 Bingo。支持自定义种子分享与难度提示。'
		}
	};
};

