# Kairem-Xtower

基于 Rust(WASM) + SvelteKit 的逻辑解谜游戏引擎与网页端原型。

## 目录结构

- `crates/engine/`：Rust 引擎库（`cdylib` + `rlib`），通过 `wasm-bindgen` 暴露 WASM 接口
- `apps/web/`：SvelteKit 前端原型（Rosemi DevTools 风格）
- `rules.json`：规则说明与颜色编码（供前端展示）

## Rust（本地调试）

```bash
cargo test -p kairm_engine
cargo run -p kairm_engine --bin debug -- --seed 123
```

## WASM 构建（给前端使用）

前端通过 `wasm-pack` 生成绑定代码到 `apps/web/src/lib/wasm/pkg/`：

```bash
cd apps/web
pnpm install
pnpm rebuild esbuild
pnpm wasm:build
pnpm dev
```
