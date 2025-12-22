import { loadEngine, type Engine } from './load';

let engine: Engine | null = null;

async function init() {
	if (!engine) {
		engine = await loadEngine();
	}
	return engine;
}

self.onmessage = async (e: MessageEvent) => {
	const { type, payload, id } = e.data;
	const eng = await init();

	try {
		let result;
		switch (type) {
			case 'generate_puzzle':
				result = eng.generate_puzzle(payload.seed, payload.size);
				break;
			case 'difficulty_report':
				result = eng.difficulty_report(payload.color_grid);
				break;
			case 'solution_count':
				result = eng.solution_count(payload.color_grid, payload.limit);
				break;
			default:
				throw new Error(`Unknown task type: ${type}`);
		}
		self.postMessage({ id, result });
	} catch (error) {
		self.postMessage({ id, error: String(error) });
	}
};