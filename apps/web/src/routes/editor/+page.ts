export const prerender = true;

import type { PageLoad } from './$types';

export const load: PageLoad = () => {
	return {
		seo: {
			title: '关卡编辑器｜聪明Bingo',
			description:
				'创建、导入、导出并试玩自制关卡：支持 URL 分享、关卡 JSON、以及解唯一性检测（limit=2）。'
		}
	};
};

