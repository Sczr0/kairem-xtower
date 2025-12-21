# Kairem-Xtower：功能现状与后续路线图（基于当前仓库实现）

> 执行者：Codex  
> 日期：2025-12-22  
> 适用范围：本仓库 `crates/engine`（Rust/WASM） + `apps/web`（SvelteKit，adapter-static）  

本文档目标：基于**当前已实现的功能**，沉淀“数据/接口契约 + 路由/URL 协议 + 存档协议”，并给出下一阶段的建议优先级与验收标准。

---

## 0) 项目概览（现状）

- 产品形态：纯前端（无后端），WASM 引擎 + 静态站点部署。
- 关卡数据：
  - `color_grid`：固定 5x5（25 格）颜色数组，前端以 `Uint8Array(25)` 传入引擎（row-major）。
  - `checked_mask`：25 位 bitmask（u32），bit i 表示第 i 格是否勾选。
- 现有路由：
  - `/`：玩法页（支持 seed / 自定义 level / 往期 daily）
  - `/editor`：关卡编辑器（导入/导出/试玩/唯一性检测）
  - `/daily`：每日题日历（基于本地存档展示通关情况）

---

## 1) 引擎能力与 WASM 接口契约（现状）

WASM 导出（`crates/engine/src/lib.rs`）：

- 题目/日期
  - `date_to_seed_ymd(date_ymd: &str) -> u64`（以 1970-01-01 为 day0）
  - `generate_puzzle(seed: u64) -> number[5][5]`
- 校验/难度/提示
  - `validate_state(checked_mask: u32, color_grid: Uint8Array(25)) -> ValidateResult`
  - `difficulty_report(color_grid: Uint8Array(25)) -> DifficultyReport`
  - `hint_next(checked_mask: u32, color_grid: Uint8Array(25)) -> HintResult`
- 解数量（编辑器/质量闸门）
  - `solution_count(color_grid, limit) -> { count, truncated }`
  - `solution_count_with_checked(checked_mask, color_grid, limit) -> { count, truncated }`
  - 约定：`limit=2` 用于区分无解/唯一/多解（>=2），性能稳定。

---

## 2) 前端路由与 URL 协议（现状）

玩法页 `/` 支持的 query 参数（`apps/web/src/routes/+page.svelte`）：

- `?level=<code>`：自定义关卡（优先级最高）
- `?seed=<u64>`：以 seed 生成题目
- `?date=YYYY-MM-DD`：打开指定日期的每日题（内部用 `date_to_seed_ymd` 得到 seed 再生成）
- 默认：今日每日题（以 `Asia/Shanghai` 时区计算“今日日期字符串”）

解析优先级：`level` > `seed` > `date` > 默认今日。

日历页 `/daily`：

- 点击日期会跳转到玩法页并携带 `?date=YYYY-MM-DD`（用于往期回看与继续挑战）。

编辑器 `/editor`：

- 支持导入 “level code / JSON / URL（含 ?level=）”，并可生成试玩链接（指向玩法页 `?level=`）。

---

## 3) 本地存档协议（现状）

存档实现位于 `apps/web/src/lib/progress.js`，核心约束：

- `localStorage` key：`kairem.progress.v1`
- 条目 key（统一抽象）：
  - `daily:YYYY-MM-DD`
  - `seed:<u64>`
  - `level:<levelCode>`
- 上限：
  - 最多保留 30 条进度（按 `updatedAt` 裁剪）
  - 撤销栈上限 200（`undo/redo`）

条目字段（`apps/web/src/lib/progress.d.ts`）：

- 核心状态：`checkedMask`、`undo[]`、`redo[]`
- 推理辅助：`marks?: number[25]`（0/1/2：无/排除/问号）
- 统计：`moveCount?`、`hintCount?`、`timeMs?`
- 完成态：`solvedAt?`（ISO 字符串）

说明：

- 黑格默认勾选：恢复/撤销时会强制 OR 上黑格 mask，避免出现“黑格被取消”的非法状态。
- 用时 `timeMs`：玩法页会在页面隐藏（`visibilitychange`）时结算并写回；通关时冻结。

---

## 4) 已实现功能清单（对应原建议优先级 1~5）

### 4.1 ✅ 优先级 1：解唯一性检测（limit=2）+ 编辑器导入导出（可玩内容闭环）

- 编辑器：`apps/web/src/routes/editor/+page.svelte`
- 关卡编码：`apps/web/src/lib/level-code.js`（配套单测：`apps/web/test/level-code.test.mjs`）
- 引擎解数：`crates/engine/src/lib.rs`（`solution_count*`）

### 4.2 ✅ 优先级 2：存档/历史 + 撤销/重做

- 存档/历史：`apps/web/src/lib/progress.js`
- 玩法页接入：`apps/web/src/routes/+page.svelte`
  - 撤销/重做按钮 + `Ctrl/Cmd+Z`、`Ctrl/Cmd+Shift+Z`
  - 历史列表（打开/删除/清空/重置）

### 4.3 ✅ 优先级 3：标记系统 + 提示解释（低成本版）

- 标记：
  - 状态工具：`apps/web/src/lib/marks.js`
  - 交互：右键循环、触屏长按、键盘 `M`（玩法页）`apps/web/src/lib/components/Matrix.svelte`
- 提示解释（低成本前端版）：`apps/web/src/routes/+page.svelte`
  - 将提示与“颜色→规则”映射关联，解释“安全提示/建议”的语义。

### 4.4 ✅ 优先级 4：每日题日历 / 往期回看

- 日历页：`apps/web/src/routes/daily/+page.svelte`
- 往期打开：玩法页 `?date=YYYY-MM-DD`（`apps/web/src/routes/+page.svelte`）
- 通关记录：`solvedAt`、`hintCount`、`timeMs`

### 4.5 ✅ 优先级 5：可访问性/色盲 + PWA/离线

- 键盘可玩性（棋盘）：
  - 方向键移动焦点、Space/Enter 操作、focus-visible 样式
  - `apps/web/src/lib/components/Matrix.svelte`
- 色盲模式（字母叠加）：
  - 设置：`apps/web/src/lib/a11y.ts`
  - 开关组件：`apps/web/src/lib/components/A11yToggle.svelte`
  - 首屏同步避免闪烁：`apps/web/src/app.html`
- PWA/离线：
  - service worker：`apps/web/src/service-worker.ts`
  - 注册（生产环境）：`apps/web/src/hooks.client.ts`
  - manifest/headers：`apps/web/static/site.webmanifest`、`apps/web/static/_headers`

---

## 5) 已知限制与建议改进点（现状）

- 提示解释目前为“低成本版”：没有结构化 reason/受影响格高亮，教学性有限（建议见 6.2）。
- PWA 离线策略为“应用壳优先”：离线时导航会回退到缓存的 `/`，不保证深链接（含 query）完全还原。
- manifest icon 仅提供 SVG：部分平台更偏好多尺寸 PNG（建议见 6.4）。

---

## 6) 下一阶段建议优先级（基于收益/成本，按当前现状重新排序）

### 6.1 统计与成就（轻量、无后端）【建议优先级：1】

目标：让“每天玩一把”的动机更强，并能在本地形成可视化反馈。

建议内容：

- 新增 `/stats`（或扩展 `/daily`）：
  - 最近 7/30 天通关趋势：用时/提示数/步数
  - 连续通关（streak）、最快用时、最少提示
  - 自定义关卡/seed 的完成率与分布

验收标准：

- 不引入后端；完全基于 `kairem.progress.v1` 汇总。
- 有明确的“数据更新时间”和“清空统计”入口（避免用户误解）。

### 6.2 提示解释（高质量引擎增强）【建议优先级：2】

目标：把提示变成“可解释的推理”，而不是“下一步答案”。

建议内容：

- 引擎 `hint_next` 返回结构化 `reason`：
  - `kind`：propagate / contradiction / goal（示例）
  - `ruleId?`：对应规则
  - `affectedCells?`：受影响格（用于高亮）
- 前端：
  - 高亮 `affectedCells`
  - 自动定位对应规则卡片，并提供“展开/折叠”提示详情

验收标准：

- 不增加明显性能退化（仍以 `limit=2` 类枚举约束为主）。
- reason 字段缺失时仍可降级显示（兼容旧版本存档/旧 wasm）。

### 6.3 新手引导/交互说明（首次进入）【建议优先级：3】

目标：降低首次上手成本，减少“看不懂规则”的流失。

建议内容：

- 首次进入弹出简短引导（可关闭、可在设置中重新打开）：
  - 勾选/提示/撤销/标记/日历入口
  - “安全提示 vs 建议”的区别

### 6.4 PWA 体验补强（安装/更新/图标）【建议优先级：4】

建议内容：

- 增加多尺寸 PNG icon（192/512 等）并写入 manifest
- 显示“新版本可用”提示（当 SW 更新时）
- 离线提示页（而不是统一回退到 `/`）

---

## 7) 验证方式（现状）

- 引擎单测：`cargo test`
- 前端单测：`pnpm -C apps/web test`
- 类型检查：`pnpm -C apps/web check`
- 生产构建（含 wasm）：`pnpm -C apps/web build`
