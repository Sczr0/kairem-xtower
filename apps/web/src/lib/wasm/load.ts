export type ValidateResult = {
	is_bingo: boolean;
	is_valid: boolean;
	cell_ok: boolean[];
	cell_messages: (string | undefined)[];
};

export type DifficultyStats = {
	node_visits: number;
	decision_points: number;
	branch_attempts: number;
	dead_ends: number;
	solutions: number;
	propagate_rounds: number;
	assignments_initial: number;
	assignments_guess: number;
	assignments_propagated: number;
	max_depth: number;
};

export type DifficultyReport = {
	difficulty_score: number;
	stats: DifficultyStats;
};

export type HintAction = 'check' | 'uncheck';

export type HintMove = {
	cell: number;
	action: HintAction;
	forced: boolean;
};

export type HintStatus = 'no_solution' | 'forced' | 'suggested';

export type HintReasonKind = 'propagate' | 'contradiction' | 'suggest' | 'repair';

export type HintReasonStep = {
	title: string;
	ruleId?: string;
	cells?: number[];
};

export type HintReason = {
	kind: HintReasonKind;
	ruleId?: string;
	affectedCells?: number[];
	secondaryCells?: number[];
	steps?: HintReasonStep[];
};

export type HintResult = {
	status: HintStatus;
	message: string;
	move?: HintMove;
	reason?: HintReason;
};

export type SolutionCountResult = {
	count: number;
	truncated: boolean;
};

export type Engine = {
	date_to_seed_ymd(date: string): bigint;
	generate_puzzle(seed: bigint): number[][];
	validate_state(checked_mask: number, color_grid: Uint8Array): ValidateResult;
	difficulty_report(color_grid: Uint8Array): DifficultyReport;
	hint_next(checked_mask: number, color_grid: Uint8Array): HintResult;
	solution_count(color_grid: Uint8Array, limit: number): SolutionCountResult;
	solution_count_with_checked(
		checked_mask: number,
		color_grid: Uint8Array,
		limit: number
	): SolutionCountResult;
};

// 说明：
// - Rust 的 u64 在 wasm-bindgen 中会映射为 JS BigInt，因此这里使用 bigint。
export async function loadEngine(): Promise<Engine> {
	try {
		// wasm-pack 输出会生成 `pkg/` 目录（本仓库默认不提交产物）。
		// @ts-ignore pkg 由 wasm-pack 生成
		const mod = await import('$lib/wasm/pkg/kairm_engine.js');
		// 优先以 URL 方式加载 .wasm，确保网络面板能看到 wasm 请求（便于排查）
		let wasmUrl: string | null = null;
		try {
			// Vite 资产导入：返回最终构建后的静态资源 URL
			wasmUrl = (await import('$lib/wasm/pkg/kairm_engine_bg.wasm?url')).default as string;
		} catch {}
		const init = (mod as any).default;
		if (typeof init === 'function') {
			// 显式传入 URL，避免 bundler 内部定位失败导致未发起 wasm 请求
			if (wasmUrl) await init(wasmUrl);
			else await init();
		}
		return mod as Engine;
	} catch (e) {
		throw new Error(
			`WASM 引擎未构建或加载失败：${String(e)}\\n\\n请先执行：\\n  pnpm wasm:build\\n再启动开发服务器。`
		);
	}
}
