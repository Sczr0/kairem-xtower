export const prerender = true;

import type { PageLoad } from './$types';

export const load: PageLoad = () => {
	return {
		seo: {
			title: '每日题日历｜聪明Bingo',
			description: '查看往期每日题并继续挑战：按日期打开关卡，查看通关时间、提示次数等记录（本地保存）。'
		}
	};
};

