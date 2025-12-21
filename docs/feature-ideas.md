# Kairem-Xtower：可新增功能与实现方法（基于现有代码）

> 执行者：Codex  
> 日期：2025-12-21  
> 适用范围：本仓库 `crates/engine`（Rust/WASM） + `apps/web`（SvelteKit）

## 0. 现有能力（可直接复用）

- 关卡/题目数据模型：固定 5x5（25 格）的颜色数组（前端扁平化后以 `Uint8Array(25)` 传入引擎）。
- 引擎（WASM）已有接口：生成题面、校验、难度评估、提示一步：
  - `generate_puzzle(seed)`
  - `validate_state(checked_mask, color_grid)`
  - `difficulty_report(color_grid)`
  - `hint_next(checked_mask, color_grid)`
- 前端已有主要交互组件：棋盘 `Matrix.svelte`、规则说明 `RuleCard.svelte`、以及玩法页 `routes/+page.svelte`。

这些能力意味着：只要能在前端构造/编辑 `color_grid` 与 `checked_mask`，就能实现“编辑器、校验、难度展示、提示”等大部分体验。

---

## 1) 关卡编辑器（核心增强）

### 用户价值

- 玩家可自制关卡并分享；也可作为开发/运营的出题工具。

### 实现方法（推荐最小闭环）

- 前端新增路由：`/editor`（SvelteKit `apps/web/src/routes/editor/+page.svelte`）。
- 复用棋盘组件：给 `Matrix.svelte` 增加“编辑模式”（不影响原玩法模式）：
  - 点击格子：在“颜色调色板”中选择颜色后写入 `grid[i]`。
  - 右键/长按：快速切换黑格/白格（可选）。
  - 继续复用 `validate_state/difficulty_report/hint_next` 做实时反馈（编辑器里提示主要用于验证出题是否合理）。
- 导入/导出：
  - `关卡 JSON`：`{ version: 1, grid: number[25] }`（或 `number[5][5]`）。
  - `URL 分享`：将 `grid` 做 base64（或更简单：压缩成字符串）放在 `?level=...`。
- 出题质量闸门：
  - 显示 `difficulty_report`（难度预估）。
  - 显示 `validate_state` 在 `checkedMask=黑格默认勾选` 时的基本合法性。
  -（见第 2 点）加入“唯一解/多解检测”，避免发布坏题。

### 需要新增的引擎能力（可选）

- 无需新增即可做“编辑 + 基础校验 + 难度 + 提示”。
- 如果要“自动生成并满足某难度目标”，可在 Rust 侧扩展 `generate` 策略并暴露更多参数（如白格比例、目标难度区间）。

---

## 2) 解数量 / 唯一解检测（强烈建议与编辑器绑定）

### 用户价值

- 提示玩家“唯一解/多解/无解”；编辑器里可做发布前检查。

### 实现方法

- Rust 引擎新增 WASM 导出：`solution_count(color_grid, limit)`（以及可选 `solution_count_with_checked(checked_mask, color_grid, limit)`）。
  - 内部调用求解器枚举：`Solver::solve_masks_limit(limit)`。
  - 返回结构建议：`{ count: number, truncated: boolean }`：
    - `limit=2`：足以区分 `0/1/>=2`（最实用、性能可控）。
    - `limit=100`：用于“显示大概解数（上限）”，避免全量枚举卡死。
- 前端展示策略：
  - 玩法页：只显示 `唯一解/多解/无解`（不追求精确数量）。
  - 编辑器：显示 `count` 与 `truncated`（例如“≥100”）。

---

## 3) 存档/历史记录/继续上次

### 用户价值

- 解决“刷新丢进度”；提供“我玩过哪些 seed/daily”。

### 实现方法

- 本地存储（无需后端）：
  - Key：`kairem:v1:puzzle:${kind}:${seed_or_date}`。
  - Value：`{ checkedMask, marks?, updatedAt }`。
- 在 `routes/+page.svelte`：
  - `onMount` 时加载存档并恢复 `checkedMask`。
  - 每次 `toggle()` 后节流写入（例如 300ms）。
- 历史列表：
  - 维护 `kairem:v1:history` 数组（存最近 N 条）。

---

## 4) 撤销/重做（Undo/Redo）

### 用户价值

- 逻辑推理类的“试探”更顺手，降低挫败。

### 实现方法

- 前端维护历史栈：
  - `past: State[]`、`present: State`、`future: State[]`。
  - `State` 至少含 `checkedMask`；若实现“标记系统”（第 5 点），也纳入状态。
- UI：
  - 按钮：撤销/重做；快捷键：`Ctrl+Z / Ctrl+Y`。

---

## 5) 标记系统（排除/怀疑/笔记）

### 用户价值

- 与“只勾选/不勾选”的二值相比，标记能显著提升可玩性与可控性。

### 实现方法（不改引擎也能做）

- 额外状态：`marks: number[25]`（例如 0=无，1=排除，2=问号）。
- `Matrix.svelte` 在 cell 内增加角标/图层渲染（仅 UI，不传引擎）。
- 交互建议：
  - 轻点：勾选/取消勾选（现有）。
  - 长按/右键：循环切换标记（排除/问号/无）。

---

## 6) 提示解释（“为什么必须这样”）

### 用户价值

- 提示更像“教学”而不是“答案”；用户更愿意继续玩。

### 实现方法（两档方案）

1) 低成本（仅前端）：在提示文案中补充“与哪条规则更相关”的解释（例如：根据该格颜色对应规则推断）。
2) 高质量（引擎增强）：为 `hint_next` 增加 `reason` 字段（结构化）：
   - 例如：`{ kind: "propagate" | "contradiction", rule?: "Blue", affectedCells?: number[] }`
   - 前端将 `affectedCells` 高亮，并在右侧规则面板定位对应规则卡片。

---

## 7) 每日题日历 / 往期回看

### 用户价值

- 增加留存：补做过的题、追连续打卡。

### 实现方法

- 新增路由：`/daily` 展示日历；点击日期 → 通过引擎 `date_to_seed_ymd(YYYY-MM-DD)` 进入该日题。
- 将完成状态写入本地存储：`{ solvedAt, timeMs, hintCount }`。

---

## 8) 统计与成就（轻量版，无后端）

### 用户价值

- 让“每天玩一把”的动力更强。

### 实现方法

- 本地汇总数据：
  - 玩法页记录：用时、步数（toggle 次数）、提示次数、是否通关。
  - 日历页/个人页汇总展示：最近 7 天趋势、平均难度、胜率等。

---

## 9) 可访问性与移动端体验增强

### 用户价值

- 触屏与键盘用户体验更好；覆盖更多人群（含色盲）。

### 实现方法

- 键盘导航：
  - `Matrix.svelte` 支持方向键移动焦点、空格切换勾选。
- 色盲模式：
  - 为颜色提供“纹理/图案/字母缩写”叠加层，而不仅靠色彩区分。
- 触控优化：
  - 长按呼出标记菜单；扩大可点击区域；减少误触。

---

## 10) PWA/离线（可选）

### 用户价值

- 可安装到桌面；断网也能打开最近题。

### 实现方法

- SvelteKit 增加 `manifest.webmanifest`（放 `apps/web/static/`）与 service worker（`apps/web/src/service-worker.ts`）。
- 缓存策略：
  - 静态资源（`/_app/*`）走 cache-first。
  - 关卡数据（seed/daily 其实可本地生成）只需缓存用户存档即可。

---

## 11) 建议优先级（按“收益/成本”排序）

1. 解唯一性检测（limit=2）+ 编辑器导入导出（形成“可玩内容闭环”）
2. 存档/历史 + 撤销/重做（显著提升日常体验）
3. 标记系统 + 提示解释（提升推理类核心手感）
4. 日历/成就（留存与运营）
5. 无障碍/色盲 + PWA（覆盖面与产品完成度）

