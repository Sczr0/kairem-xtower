export type ValidateResult = {
	is_bingo: boolean;
	is_valid: boolean;
	cell_ok: boolean[];
	cell_messages: (string | undefined)[];
};

export type Engine = {
	date_to_seed_ymd(date: string): bigint;
	generate_puzzle(seed: bigint): number[][];
	validate_state(checked_mask: number, color_grid: Uint8Array): ValidateResult;
};

// 说明：
// - Rust 的 u64 在 wasm-bindgen 中会映射为 JS BigInt，因此这里使用 bigint。
export async function loadEngine(): Promise<Engine> {
	try {
		// wasm-pack 输出会生成 `pkg/` 目录（本仓库默认不提交产物）。
		// @ts-ignore pkg 由 wasm-pack 生成
		const mod = await import('$lib/wasm/pkg/kairm_engine');
		return mod as Engine;
	} catch (e) {
		throw new Error(
			`WASM 引擎未构建或加载失败：${String(e)}\\n\\n请先执行：\\n  pnpm wasm:build\\n再启动开发服务器。`
		);
	}
}
