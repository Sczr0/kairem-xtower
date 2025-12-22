export const prerender = true;

import type { PageLoad } from './$types';

export const load: PageLoad = () => {
	return {
		seo: {
			title: '离线提示｜聪明Bingo',
			description: '你当前处于离线状态：可稍后重试，或返回已缓存的页面继续游玩。'
		}
	};
};

