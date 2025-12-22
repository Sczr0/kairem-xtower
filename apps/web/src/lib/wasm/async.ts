import type { SolutionCountResult, DifficultyReport } from './load';

let worker: Worker | null = null;
let nextId = 0;
const pending = new Map<number, { resolve: (v: any) => void; reject: (e: any) => void }>();

function getWorker() {
	if (!worker) {
		worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
		worker.onmessage = (e) => {
			const { id, result, error } = e.data;
			const p = pending.get(id);
			if (p) {
				pending.delete(id);
				if (error) p.reject(error);
				else p.resolve(result);
			}
		};
	}
	return worker;
}

function callWorker<T>(type: string, payload: any): Promise<T> {
	const id = nextId++;
	return new Promise((resolve, reject) => {
		pending.set(id, { resolve, reject });
		getWorker().postMessage({ type, payload, id });
	});
}

export async function generatePuzzleAsync(seed: bigint, size: number = 5): Promise<number[][]> {
	return callWorker('generate_puzzle', { seed, size });
}

export async function getDifficultyReportAsync(colorGrid: Uint8Array): Promise<DifficultyReport> {
	return callWorker('difficulty_report', { color_grid: colorGrid });
}

export async function getSolutionCountAsync(
	colorGrid: Uint8Array,
	limit: number
): Promise<SolutionCountResult> {
	return callWorker('solution_count', { color_grid: colorGrid, limit });
}

export async function getSolutionCountWithCheckedAsync(
	checkedMask: bigint,
	colorGrid: Uint8Array,
	limit: number
): Promise<SolutionCountResult> {
	return callWorker('solution_count_with_checked', {
		checked_mask: checkedMask,
		color_grid: colorGrid,
		limit
	});
}