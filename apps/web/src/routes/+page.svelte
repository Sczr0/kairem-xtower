<svelte:options runes={false} />

<script lang="ts">
	import Matrix from '$lib/components/Matrix.svelte';
	import RuleCard, { type Rule as UiRule } from '$lib/components/RuleCard.svelte';
	import ThemeToggle from '$lib/components/ThemeToggle.svelte';
	import A11yToggle from '$lib/components/A11yToggle.svelte';
	import rules from '$lib/rules.json';
	import { decodeLevel, encodeLevel } from '$lib/level-code.js';
	import {
		clearAllProgress,
		createHistory,
		deleteProgressEntry,
		HISTORY_LIMIT,
		historyPush,
		historyRedo,
		historyUndo,
		loadProgressEntry,
		listProgressEntries,
		makePuzzleKey,
		normalizeHistory,
		normalizeMaskU32,
		upsertProgressEntry,
		type HistoryState,
		type ProgressEntry
	} from '$lib/progress.js';
	import { createMarks, cycleMarkValue, normalizeMarks } from '$lib/marks.js';
	import { Color, type ColorId, colorToCss } from '$lib/colors';
	import {
		loadEngine,
		type DifficultyReport,
		type Engine,
		type HintResult,
		type ValidateResult
	} from '$lib/wasm/load';
	import { browser, dev } from '$app/environment';
	import { onMount, tick } from 'svelte';
	import { fade, slide } from 'svelte/transition';
	import { colorBlindEnabled } from '$lib/a11y';
	import { readTutorialDismissedAt, setTutorialDismissed, shouldAutoShowTutorial } from '$lib/tutorial.js';

	// --- 逻辑部分保持不变 ---
	let engine: Engine | null = null;
	let engineError = '';

	let puzzleKind: 'daily' | 'seed' | 'custom' = 'daily';
	let urlSeedError = '';
	let urlLevelError = '';

	let dateYmd = '';
	let seed: bigint | null = null;

	let shareToast = '';
	let shareUrlForManualCopy = '';
	let shareManualVisible = false;

	let tutorialOpen = false;
	let tutorialDontAutoShow = true;
	let tutorialDismissedAt: string | null = null;
	let tutorialDialogEl: HTMLDivElement | null = null;
	let tutorialLastActiveEl: HTMLElement | null = null;
	let tutorialBodyOverflowBefore: string | null = null;

	let grid2d: number[][] = [];
	let grid: ColorId[] = Array.from({ length: 25 }, () => Color.White);
	let checkedMask = 0;
	let marks: number[] = createMarks();
	let levelCode: string | null = null;
	let history: HistoryState = createHistory(0);
	let moveCount = 0;
	let hintCount = 0;
	let timeMs = 0;
	let timerStartedAt: number | null = null;
	let solvedAt: string | null = null;
	let progressEntries: ProgressEntry[] = [];
	let keyboardBound = false;
	let validate: ValidateResult | null = null;
	let difficulty: DifficultyReport | null = null;
	let hoveredRuleId: string | null = null;
	let activeCellIndex: number | null = null;
	let allRulesOpen = false;
	let hint: HintResult | null = null;
	let hintIndex: number | null = null;
	let hintAction: 'check' | 'uncheck' | null = null;
	let hintLoading = false;
	let hintExplain: string | null = null;
	let hintExplainCells: number[] = [];
	let hintExplainSecondaryCells: number[] = [];
	let hintExplainDetailsOpen = false;
	let totalTimeMs = 0;
	let clockTick = 0;
	let allRulesDetailsEl: HTMLDetailsElement | null = null;
	let ruleCardEls: Record<string, HTMLElement | null> = {};

	function ruleRef(node: HTMLElement, id: string) {
		ruleCardEls[id] = node;
		return {
			destroy() {
				if (ruleCardEls[id] === node) delete ruleCardEls[id];
			}
		};
	}

	type Rule = UiRule;
	const allRules = (rules.rules ?? []) as Rule[];
	const goalRule = allRules.find((r) => r.id === 'bingo') ?? null;

	const ruleColorMap: Partial<Record<string, ColorId>> = {
		red: Color.Red,
		blue: Color.Blue,
		green: Color.Green,
		yellow: Color.Yellow,
		purple: Color.Purple,
		orange: Color.Orange,
		cyan: Color.Cyan,
		black: Color.Black
	};

	// 反向映射：ColorId -> RuleId
	const colorRuleMap = Object.entries(ruleColorMap).reduce(
		(acc, [ruleId, colorId]) => {
			if (colorId !== undefined) {
				acc[colorId] = ruleId;
			}
			return acc;
		},
		{} as Record<number, string>
	);

	function ruleColorCss(ruleId: string): string {
		if (ruleId === 'bingo') return '#f59e0b';
		const colorId = ruleColorMap[ruleId];
		if (colorId === undefined) return '#94a3b8';
		return colorToCss(colorId);
	}

	$: hoveredRule = hoveredRuleId ? allRules.find((r) => r.id === hoveredRuleId) ?? null : null;
	$: isSolved = !!(validate?.is_valid && validate?.is_bingo);
	$: canUndo = history.undo.length > 0;
	$: canRedo = history.redo.length > 0;
	$: hasMarks = marks.some((v) => v !== 0);
	$: hintExplain = buildHintExplain(hint);
	$: currentProgressKey = safeCurrentPuzzleKey();
	$: totalTimeMs = snapshotTimeMs(clockTick || Date.now());

	function snapshotTimeMs(now = Date.now()): number {
		if (timerStartedAt === null) return Math.max(0, timeMs);
		const delta = now - timerStartedAt;
		if (!Number.isFinite(delta) || delta <= 0) return Math.max(0, timeMs);
		return Math.max(0, timeMs + delta);
	}

	function shanghaiDateYmd(now = new Date()): string {
		const fmt = new Intl.DateTimeFormat('zh-CN', {
			timeZone: 'Asia/Shanghai',
			year: 'numeric',
			month: '2-digit',
			day: '2-digit'
		});
		const parts = fmt.formatToParts(now);
		const y = parts.find((p) => p.type === 'year')?.value ?? '1970';
		const m = parts.find((p) => p.type === 'month')?.value ?? '01';
		const d = parts.find((p) => p.type === 'day')?.value ?? '01';
		return `${y}-${m}-${d}`;
	}

	function parseSeed(raw: string): bigint | null {
		const s = raw.trim();
		if (!s) return null;
		try {
			const v = BigInt(s);
			if (v < 0n) return null;
			return v;
		} catch {
			return null;
		}
	}

	function parseDateYmd(raw: string): string | null {
		const s = raw.trim();
		if (!s) return null;
		if (!/^\d{4}-\d{2}-\d{2}$/.test(s)) return null;
		const parts = s.split('-').map((x) => Number(x));
		if (parts.length !== 3) return null;
		const [y, m, d] = parts;
		if (!Number.isFinite(y) || !Number.isFinite(m) || !Number.isFinite(d)) return null;
		if (m < 1 || m > 12) return null;
		if (d < 1 || d > 31) return null;
		return s;
	}

	function shortSeed(v: bigint): string {
		const s = v.toString();
		if (s.length <= 12) return s;
		return `${s.slice(0, 6)}…${s.slice(-4)}`;
	}

	function shortLevelCode(code: string): string {
		const s = String(code ?? '').trim();
		if (!s) return '--';
		if (s.length <= 18) return s;
		return `${s.slice(0, 10)}…${s.slice(-6)}`;
	}

	function formatDuration(ms: number): string {
		const t = Math.max(0, Math.floor(ms / 1000));
		const s = t % 60;
		const m = Math.floor(t / 60) % 60;
		const h = Math.floor(t / 3600);
		const pad2 = (v: number) => String(v).padStart(2, '0');
		if (h > 0) return `${h}:${pad2(m)}:${pad2(s)}`;
		return `${m}:${pad2(s)}`;
	}

	function safeLevelCodeForDisplay(): string {
		try {
			return levelCode ?? encodeLevel(grid);
		} catch {
			return '--';
		}
	}

	function buildSeedUrl(v: bigint): string {
		if (!browser) return '';
		const url = new URL(window.location.href);
		url.searchParams.delete('date');
		url.searchParams.delete('level');
		url.searchParams.set('seed', v.toString());
		return url.toString();
	}

	function replaceUrlSeed(v: bigint | null) {
		if (!browser) return;
		const url = new URL(window.location.href);
		url.searchParams.delete('date');
		url.searchParams.delete('level');
		if (v === null) url.searchParams.delete('seed');
		else url.searchParams.set('seed', v.toString());
		window.history.replaceState({}, '', url.toString());
	}

	function buildLevelUrl(code: string): string {
		if (!browser) return '';
		const url = new URL(window.location.href);
		url.searchParams.delete('date');
		url.searchParams.delete('seed');
		url.searchParams.set('level', code);
		return url.toString();
	}

	function replaceUrlLevel(code: string | null) {
		if (!browser) return;
		const url = new URL(window.location.href);
		url.searchParams.delete('date');
		url.searchParams.delete('seed');
		if (code === null) url.searchParams.delete('level');
		else url.searchParams.set('level', code);
		window.history.replaceState({}, '', url.toString());
	}

	function buildDailyUrl(date: string): string {
		if (!browser) return '';
		const url = new URL(window.location.href);
		url.searchParams.delete('seed');
		url.searchParams.delete('level');
		url.searchParams.set('date', date);
		return url.toString();
	}

	function replaceUrlDaily(date: string | null) {
		if (!browser) return;
		const url = new URL(window.location.href);
		url.searchParams.delete('seed');
		url.searchParams.delete('level');
		if (!date) url.searchParams.delete('date');
		else url.searchParams.set('date', date);
		window.history.replaceState({}, '', url.toString());
	}

	function randomSeedU64(): bigint {
		if (browser && typeof window.crypto?.getRandomValues === 'function') {
			const buf = new Uint32Array(2);
			window.crypto.getRandomValues(buf);
			return (BigInt(buf[0]) << 32n) | BigInt(buf[1]);
		}
		const hi = Math.floor(Math.random() * 0x1_0000_0000);
		const lo = Math.floor(Math.random() * 0x1_0000_0000);
		return (BigInt(hi) << 32n) | BigInt(lo);
	}

	function showToast(msg: string) {
		shareToast = msg;
		setTimeout(() => {
			if (shareToast === msg) shareToast = '';
		}, 2000);
	}

	function clearHint() {
		hint = null;
		hintIndex = null;
		hintAction = null;
		hintExplainCells = [];
		hintExplainSecondaryCells = [];
		hintExplainDetailsOpen = false;
	}

	function lockBodyScrollForTutorial() {
		if (!browser) return;
		if (tutorialBodyOverflowBefore === null) {
			tutorialBodyOverflowBefore = document.body.style.overflow ?? '';
		}
		document.body.style.overflow = 'hidden';
	}

	function unlockBodyScrollForTutorial() {
		if (!browser) return;
		if (tutorialBodyOverflowBefore !== null) {
			document.body.style.overflow = tutorialBodyOverflowBefore;
			tutorialBodyOverflowBefore = null;
		}
	}

	function restoreFocusAfterTutorial() {
		if (!browser) return;
		const el = tutorialLastActiveEl;
		tutorialLastActiveEl = null;
		el?.focus?.();
	}

	function cleanupTutorialUi() {
		unlockBodyScrollForTutorial();
		restoreFocusAfterTutorial();
	}

	function onTutorialKeydown(event: KeyboardEvent) {
		if (!tutorialOpen) return;

		if (event.key === 'Escape') {
			event.preventDefault();
			closeTutorialSessionOnly();
			return;
		}

		if (event.key !== 'Tab') return;
		if (!tutorialDialogEl) return;

		const focusables = Array.from(
			tutorialDialogEl.querySelectorAll<HTMLElement>(
				'a[href],button:not([disabled]),textarea,input,select,[tabindex]:not([tabindex="-1"])'
			)
		).filter((el) => {
			const style = window.getComputedStyle(el);
			return style.display !== 'none' && style.visibility !== 'hidden';
		});

		if (focusables.length === 0) {
			event.preventDefault();
			tutorialDialogEl.focus();
			return;
		}

		const first = focusables[0];
		const last = focusables[focusables.length - 1];
		const active = document.activeElement as HTMLElement | null;

		if (event.shiftKey && active === first) {
			event.preventDefault();
			last.focus();
			return;
		}
		if (!event.shiftKey && active === last) {
			event.preventDefault();
			first.focus();
		}
	}

	async function openTutorial() {
		if (browser) tutorialLastActiveEl = document.activeElement as HTMLElement | null;
		lockBodyScrollForTutorial();
		tutorialOpen = true;
		await tick();
		tutorialDialogEl?.focus();
	}

	function closeTutorial() {
		tutorialOpen = false;
		cleanupTutorialUi();
		setTutorialDismissed(!!tutorialDontAutoShow);
		tutorialDismissedAt = readTutorialDismissedAt();
	}

	function closeTutorialSessionOnly() {
		tutorialOpen = false;
		cleanupTutorialUi();
	}

	function pauseTimer() {
		if (timerStartedAt === null) return;
		timeMs = snapshotTimeMs();
		timerStartedAt = null;
	}

	function resumeTimer() {
		if (timerStartedAt !== null) return;
		if (solvedAt) return;
		timerStartedAt = Date.now();
	}

	function selectAll(event: Event) {
		const input = event.currentTarget as HTMLInputElement | null;
		input?.select();
	}

	function flattenGrid2d(g: number[][]): ColorId[] {
		return g.flat().map((x) => x as ColorId);
	}

	function blackMaskFromGrid(flat: ColorId[]): number {
		let m = 0;
		for (let i = 0; i < flat.length; i++) {
			if (flat[i] === Color.Black) m |= 1 << i;
		}
		return m >>> 0;
	}

	function safeCurrentPuzzleKey(): string | null {
		try {
			if (puzzleKind === 'daily') {
				if (!dateYmd) return null;
				return makePuzzleKey('daily', { dateYmd });
			}
			if (puzzleKind === 'seed') {
				if (!seed) return null;
				return makePuzzleKey('seed', { seed: seed.toString() });
			}
			// custom
			let code = levelCode;
			if (!code) {
				code = encodeLevel(grid);
				levelCode = code;
			}
			return makePuzzleKey('custom', { levelCode: code });
		} catch {
			return null;
		}
	}

	function refreshProgressEntries() {
		progressEntries = listProgressEntries();
	}

	function applyBlackMaskToHistory(h: HistoryState, blackMask: number): HistoryState {
		const apply = (m: number) => normalizeMaskU32((m | blackMask) >>> 0);
		return normalizeHistory(
			{
				undo: (h.undo ?? []).map(apply),
				redo: (h.redo ?? []).map(apply),
				present: apply(h.present)
			},
			HISTORY_LIMIT
		);
	}

	function persistProgress() {
		const key = safeCurrentPuzzleKey();
		if (!key) return;

		const nowIso = new Date().toISOString();
		if (isSolved && !solvedAt) {
			solvedAt = nowIso;
			if (timerStartedAt !== null) {
				timeMs = snapshotTimeMs();
				timerStartedAt = null;
			}
		}

		const kind: ProgressEntry['kind'] =
			puzzleKind === 'custom' ? 'custom' : puzzleKind === 'daily' ? 'daily' : 'seed';

		const timeMsToSave = isSolved ? Math.max(0, timeMs) : snapshotTimeMs();

		const entry: ProgressEntry = {
			key,
			kind,
			dateYmd: kind === 'daily' ? dateYmd : undefined,
			seed: seed ? seed.toString() : undefined,
			levelCode: kind === 'custom' ? (levelCode ?? encodeLevel(grid)) : undefined,
			checkedMask: checkedMask >>> 0,
			marks,
			undo: history.undo,
			redo: history.redo,
			moveCount,
			hintCount,
			timeMs: timeMsToSave,
			solvedAt: solvedAt ?? undefined
		};

		upsertProgressEntry(entry);
		refreshProgressEntries();
	}

	function restoreProgressIfAny() {
		const key = safeCurrentPuzzleKey();
		if (!key) return;

		const saved = loadProgressEntry(key);
		if (!saved) return;

		if (saved.kind === 'custom' && typeof saved.levelCode === 'string') {
			levelCode = saved.levelCode;
		}

		const blackMask = blackMaskFromGrid(grid);
		const restored = applyBlackMaskToHistory(
			normalizeHistory(
				{
					undo: saved.undo ?? [],
					redo: saved.redo ?? [],
					present: typeof saved.checkedMask === 'number' ? saved.checkedMask : blackMask
				},
				HISTORY_LIMIT
			),
			blackMask
		);

		history = restored;
		checkedMask = restored.present;
		marks = normalizeMarks(saved.marks);
		moveCount = typeof saved.moveCount === 'number' ? saved.moveCount : 0;
		hintCount = typeof saved.hintCount === 'number' ? saved.hintCount : 0;
		timeMs = typeof saved.timeMs === 'number' && Number.isFinite(saved.timeMs) ? Math.max(0, saved.timeMs) : 0;
		solvedAt = typeof saved.solvedAt === 'string' ? saved.solvedAt : null;
		timerStartedAt = solvedAt ? null : Date.now();
	}

	function refreshValidate() {
		if (!engine) return;
		validate = engine.validate_state(checkedMask >>> 0, new Uint8Array(grid));
	}

	function refreshDifficulty() {
		if (!engine) return;
		try {
			difficulty = engine.difficulty_report(new Uint8Array(grid));
		} catch {
			difficulty = null;
		}
	}

	function focusRuleByIndex(i: number) {
		const color = grid[i];
		hoveredRuleId = colorRuleMap[color] ?? null;
		activeCellIndex = i;
	}

	async function focusRuleById(ruleId: string | null, opts: { openAllRules?: boolean } = {}) {
		hoveredRuleId = ruleId;
		if (!ruleId) return;

		if (opts.openAllRules && allRulesDetailsEl) {
			allRulesDetailsEl.open = true;
			allRulesOpen = true;
			await tick();
			ruleCardEls[ruleId]?.scrollIntoView({ block: 'nearest' });
		}
	}

	function buildHintExplain(h: HintResult | null): string | null {
		if (!h) return null;

		if (h.status === 'no_solution') {
			return '当前状态已无解：建议先撤销最近几步，或重置进度后再尝试。';
		}

		const mv = h.move ?? null;
		if (!mv) return null;

		const ruleId = h.reason?.ruleId ?? (colorRuleMap[grid[mv.cell]] ?? null);
		const rule = ruleId ? allRules.find((r) => r.id === ruleId) ?? null : null;

		const relatedFrom = h.reason?.ruleId ? '（引擎解释）' : '（由颜色推断）';
		const related =
			rule && ruleId
				? `相关规则：${rule.name}${relatedFrom}。${rule.description}`
				: '相关规则：未知（未能映射到规则）。';

		const kind = h.reason?.kind ?? null;
		const kindExplain =
			kind === 'propagate'
				? '解释：这是由规则传播直接推出的强制结论。'
				: kind === 'contradiction'
					? '解释：这是通过反证推出的强制结论（另一种选择会导致矛盾/无解）。'
					: kind === 'repair'
						? '解释：当前状态无解，这是用于“修复回到可解状态”的建议。'
						: '解释：这是建议方向（不保证唯一，但通常能推进推理）。';

		return `${related}\n${kindExplain}`;
	}

	function toggle(i: number) {
		if (grid[i] === Color.Black) return;
		const next = (checkedMask ^ (1 << i)) >>> 0;
		history = historyPush({ undo: history.undo, redo: history.redo, present: checkedMask }, next);
		checkedMask = history.present;
		moveCount += 1;
		focusRuleByIndex(i);
		refreshValidate();
		clearHint();
		persistProgress();
	}

	function cycleMark(i: number) {
		if (grid[i] === Color.Black) return;
		const next = [...marks];
		next[i] = cycleMarkValue(next[i]);
		marks = next;
		focusRuleByIndex(i);
		persistProgress();
	}

	function clearMarks() {
		marks = createMarks();
		persistProgress();
	}

	function handleHover(index: number | null) {
		if (index === null) return;
		focusRuleByIndex(index);
	}

	function undo() {
		if (!canUndo) return;
		history = historyUndo({ undo: history.undo, redo: history.redo, present: checkedMask });
		checkedMask = history.present;
		refreshValidate();
		clearHint();
		persistProgress();
	}

	function redo() {
		if (!canRedo) return;
		history = historyRedo({ undo: history.undo, redo: history.redo, present: checkedMask });
		checkedMask = history.present;
		refreshValidate();
		clearHint();
		persistProgress();
	}

	function handleGlobalKeyDown(event: KeyboardEvent) {
		if (event.defaultPrevented) return;
		const target = event.target as HTMLElement | null;
		const tag = target?.tagName?.toLowerCase() ?? '';
		if (tag === 'input' || tag === 'textarea' || (target as any)?.isContentEditable) return;

		const mod = event.ctrlKey || event.metaKey;
		if (!mod) return;
		if (event.key.toLowerCase() !== 'z') return;

		event.preventDefault();
		if (event.shiftKey) redo();
		else undo();
	}

	async function requestHint() {
		if (!engine) return;
		if (validate?.is_valid && validate?.is_bingo) {
			showToast('已通关：无需提示');
			return;
		}

		hintLoading = true;
		try {
			const res = engine.hint_next(checkedMask >>> 0, new Uint8Array(grid));
			hint = res;
			hintExplainDetailsOpen = false;
			if (Array.isArray(res.reason?.affectedCells)) {
				const raw = res.reason?.affectedCells ?? [];
				hintExplainCells = raw
					.filter((x) => typeof x === 'number' && x >= 0 && x < 25)
					.filter((x, idx, arr) => arr.indexOf(x) === idx);
			} else if (res.move) {
				hintExplainCells = [res.move.cell];
			} else {
				hintExplainCells = [];
			}

			if (Array.isArray(res.reason?.secondaryCells)) {
				const raw = res.reason?.secondaryCells ?? [];
				hintExplainSecondaryCells = raw
					.filter((x) => typeof x === 'number' && x >= 0 && x < 25)
					.filter((x, idx, arr) => arr.indexOf(x) === idx);
			} else {
				hintExplainSecondaryCells = [];
			}
			hintCount += 1;
			persistProgress();
			const mv = res.move ?? null;
			if (mv) {
				hintIndex = mv.cell;
				hintAction = mv.action;
				focusRuleByIndex(mv.cell);
				await focusRuleById(res.reason?.ruleId ?? (colorRuleMap[grid[mv.cell]] ?? null), { openAllRules: true });
			} else {
				hintIndex = null;
				hintAction = null;
			}
		} catch (e) {
			clearHint();
			showToast(`提示失败：${String(e)}`);
		} finally {
			hintLoading = false;
		}
	}

	function applyHintMove() {
		const mv = hint?.move;
		if (!mv) return;
		const i = mv.cell;
		const isChecked = ((checkedMask >>> 0) & (1 << i)) !== 0;

		if (mv.action === 'check') {
			if (isChecked) {
				showToast('该格已勾选');
				return;
			}
			toggle(i);
			return;
		}

		// uncheck
		if (!isChecked) {
			showToast('该格当前未勾选');
			return;
		}
		toggle(i);
	}

	async function loadPuzzleBySeed(newSeed: bigint, opts: { updateUrl?: boolean } = {}) {
		if (!engine) return;
		seed = newSeed;
		levelCode = null;
		grid2d = engine.generate_puzzle(seed);
		grid = flattenGrid2d(grid2d);
		checkedMask = blackMaskFromGrid(grid);
		marks = createMarks();
		history = createHistory(checkedMask);
		moveCount = 0;
		hintCount = 0;
		timeMs = 0;
		timerStartedAt = Date.now();
		solvedAt = null;
		validate = null;
		hoveredRuleId = null;
		activeCellIndex = null;
		clearHint();
		restoreProgressIfAny();
		refreshValidate();
		refreshDifficulty();
		if (opts.updateUrl) replaceUrlSeed(seed);
		persistProgress();
	}

	function loadPuzzleByCustomGrid(flat: ColorId[], opts: { updateUrl?: boolean; levelCode?: string } = {}) {
		seed = null;
		grid2d = [];
		grid = [...flat];
		checkedMask = blackMaskFromGrid(grid);
		marks = createMarks();
		levelCode = opts.levelCode ?? null;
		history = createHistory(checkedMask);
		moveCount = 0;
		hintCount = 0;
		timeMs = 0;
		timerStartedAt = Date.now();
		solvedAt = null;
		validate = null;
		hoveredRuleId = null;
		activeCellIndex = null;
		clearHint();
		restoreProgressIfAny();
		refreshValidate();
		refreshDifficulty();
		if (opts.updateUrl) {
			try {
				const code = levelCode ?? encodeLevel(grid);
				levelCode = code;
				replaceUrlLevel(code);
			} catch {
				replaceUrlLevel(null);
			}
		}
		persistProgress();
	}

	async function newSeedPuzzle(newSeed: bigint, opts: { updateUrl?: boolean } = {}) {
		if (!engine) return;
		puzzleKind = 'seed';
		urlSeedError = '';
		urlLevelError = '';
		dateYmd = '';
		await loadPuzzleBySeed(newSeed, opts);
	}

	async function newDailyPuzzle(
		opts: { updateUrl?: boolean; keepUrlSeedError?: boolean; keepUrlLevelError?: boolean } = {}
	) {
		if (!engine) return;
		puzzleKind = 'daily';
		if (!opts.keepUrlSeedError) urlSeedError = '';
		if (!opts.keepUrlLevelError) urlLevelError = '';
		dateYmd = shanghaiDateYmd();
		const dailySeed = engine.date_to_seed_ymd(dateYmd);
		await loadPuzzleBySeed(dailySeed);
		if (opts.updateUrl) replaceUrlDaily(null);
	}

	async function newRandomPuzzle() {
		if (!engine) return;
		await newSeedPuzzle(randomSeedU64(), { updateUrl: true });
	}

	async function loadDailyPuzzleByDate(targetDateYmd: string, opts: { updateUrl?: boolean } = {}) {
		if (!engine) return;
		puzzleKind = 'daily';
		urlSeedError = '';
		urlLevelError = '';
		dateYmd = targetDateYmd;
		const dailySeed = engine.date_to_seed_ymd(dateYmd);
		await loadPuzzleBySeed(dailySeed);
		if (opts.updateUrl) replaceUrlDaily(targetDateYmd);
	}

	async function openProgressEntry(entry: ProgressEntry) {
		try {
			if (!engine) return;
			if (entry.kind === 'daily' && entry.dateYmd) {
				await loadDailyPuzzleByDate(entry.dateYmd, { updateUrl: true });
				return;
			}
			if (entry.kind === 'seed' && entry.seed) {
				await newSeedPuzzle(BigInt(entry.seed), { updateUrl: true });
				return;
			}
			if (entry.kind === 'custom' && entry.levelCode) {
				const decoded = decodeLevel(entry.levelCode);
				puzzleKind = 'custom';
				urlSeedError = '';
				urlLevelError = '';
				dateYmd = '';
				loadPuzzleByCustomGrid(decoded.grid as ColorId[], {
					updateUrl: true,
					levelCode: entry.levelCode
				});
				return;
			}
			showToast('存档信息不完整，无法打开');
		} catch (e) {
			showToast(`打开存档失败：${String(e)}`);
		}
	}

	function removeProgressEntry(key: string) {
		deleteProgressEntry(key);
		refreshProgressEntries();
		if (currentProgressKey === key) {
			showToast('已删除当前关卡存档');
		}
	}

	function clearAllProgressWithConfirm() {
		if (!browser) return;
		if (!confirm('确定清空所有本地存档/历史吗？')) return;
		clearAllProgress();
		refreshProgressEntries();
		showToast('已清空');
	}

	function resetCurrentProgressWithConfirm() {
		if (!browser) return;
		const key = currentProgressKey;
		if (!key) return;
		if (!confirm('确定重置当前关卡进度吗？')) return;

		deleteProgressEntry(key);
		const blackMask = blackMaskFromGrid(grid);
		history = createHistory(blackMask);
		checkedMask = blackMask;
		marks = createMarks();
		moveCount = 0;
		hintCount = 0;
		timeMs = 0;
		timerStartedAt = Date.now();
		solvedAt = null;
		refreshValidate();
		clearHint();
		persistProgress();
		showToast('已重置');
	}

	function progressTitle(entry: ProgressEntry): string {
		if (entry.kind === 'daily') return `每日 ${entry.dateYmd ?? ''}`.trim();
		if (entry.kind === 'seed') return `seed ${entry.seed ?? ''}`.trim();
		return `自定义 ${shortLevelCode(entry.levelCode ?? '')}`.trim();
	}

	function progressBadge(entry: ProgressEntry): string {
		if (entry.solvedAt) return '已通关';
		return '进行中';
	}

	function formatTimestamp(ts?: string): string {
		if (!ts) return '';
		try {
			return new Date(ts).toLocaleString('zh-CN', { hour12: false });
		} catch {
			return '';
		}
	}

	async function sharePuzzle() {
		if (!browser) return;

		let url = '';
		if (puzzleKind === 'custom') {
			try {
				const code = levelCode ?? encodeLevel(grid);
				levelCode = code;
				url = buildLevelUrl(code);
			} catch (e) {
				showToast(`关卡编码失败：${String(e)}`);
				return;
			}
		} else {
			if (!seed) return;
			url = buildSeedUrl(seed);
		}
		shareUrlForManualCopy = url;
		shareManualVisible = false;

		if (typeof navigator.share === 'function') {
			try {
				await navigator.share({ title: 'Kairem', url });
				showToast('已打开系统分享');
				return;
			} catch {
				// 用户取消也会抛错，继续走复制兜底即可
			}
		}

		if (typeof navigator.clipboard?.writeText !== 'function') {
			shareManualVisible = true;
			showToast('当前环境不支持自动复制，请手动复制链接');
			return;
		}

		try {
			await navigator.clipboard.writeText(url);
			showToast('链接已复制');
		} catch {
			shareManualVisible = true;
			showToast('复制失败，请手动复制链接');
		}
	}

	onMount(() => {
		const onVisibility = () => {
			if (document.visibilityState === 'hidden') {
				pauseTimer();
				persistProgress();
			}
			else resumeTimer();
		};
		document.addEventListener('visibilitychange', onVisibility);

		tutorialDismissedAt = readTutorialDismissedAt();
		tutorialDontAutoShow = !shouldAutoShowTutorial(tutorialDismissedAt);
		// 首次进入自动弹出（不阻塞引擎加载）
		if (browser && shouldAutoShowTutorial(tutorialDismissedAt)) {
			setTimeout(() => {
				void openTutorial();
			}, 80);
		}

		clockTick = Date.now();
		const clock = setInterval(() => {
			clockTick = Date.now();
		}, 1000);

		(async () => {
			try {
				engine = await loadEngine();
				refreshProgressEntries();
				if (!keyboardBound) {
					window.addEventListener('keydown', handleGlobalKeyDown);
					keyboardBound = true;
				}

				const url = new URL(window.location.href);

				const rawLevel = url.searchParams.get('level');
				if (rawLevel !== null) {
					try {
						const decoded = decodeLevel(rawLevel);
						puzzleKind = 'custom';
						urlSeedError = '';
						urlLevelError = '';
						dateYmd = '';
						loadPuzzleByCustomGrid(decoded.grid as ColorId[], { updateUrl: true, levelCode: rawLevel });
						return;
					} catch {
						urlLevelError = 'level 参数无效，已回退到今日题目';
						await newDailyPuzzle({ updateUrl: true, keepUrlLevelError: true });
						return;
					}
				}

				const rawSeed = url.searchParams.get('seed');
				if (rawSeed !== null) {
					const parsed = parseSeed(rawSeed);
					if (parsed === null) {
						urlSeedError = 'seed 参数无效，已回退到今日题目';
						await newDailyPuzzle({ updateUrl: true, keepUrlSeedError: true });
						return;
					}

					await newSeedPuzzle(parsed, { updateUrl: true });
					return;
				}

				const rawDate = url.searchParams.get('date');
				if (rawDate !== null) {
					const parsed = parseDateYmd(rawDate);
					if (parsed === null) {
						urlSeedError = '';
						urlLevelError = '';
						await newDailyPuzzle({ updateUrl: true });
						showToast('date 参数无效，已回退到今日题目');
						return;
					}

					try {
						await loadDailyPuzzleByDate(parsed, { updateUrl: true });
						return;
					} catch (e) {
						await newDailyPuzzle({ updateUrl: true });
						showToast(`加载每日题失败，已回退到今日题目：${String(e)}`);
						return;
					}
				}

				await newDailyPuzzle();
			} catch (e) {
				engineError = String(e);
			}
		})();

		return () => {
			document.removeEventListener('visibilitychange', onVisibility);
			clearInterval(clock);
		};
	});
</script>

<div class="page-container">
	{#if engineError}
		<div class="error-banner">引擎加载失败：{engineError}</div>
	{:else}
		<header class="header">
            <div class="logo-area">
			    <h1 class="logo">聪明Bingo</h1>
			    <p class="subtitle">一个轻量逻辑游戏，灵感来源QQ群</p>
            </div>
			<div class="header-actions">
				<button class="btn" type="button" on:click={openTutorial} title="新手引导（可重复打开）">引导</button>
				<a class="btn" href="/daily" title="每日题日历">日历</a>
				<a class="btn" href="/stats" title="统计与成就">统计</a>
				<a class="btn" href="/editor" title="关卡编辑器">编辑器</a>
				<a
					href="https://github.com/Sczr0/kairem-xtower"
					target="_blank"
					rel="noopener noreferrer"
					class="btn github-link"
					aria-label="GitHub 仓库"
					title="GitHub 仓库"
				>
					<svg class="icon" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
						<path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
					</svg>
				</a>
				<A11yToggle />
				<ThemeToggle />
			</div>
		</header>

		<div class="main-layout">
			<!-- 左侧/上方：游戏主区域 -->
			<main class="card game-area">
                
                <!-- 工具栏：整合信息与操作 -->
				<div class="toolbar">
					<div class="game-info">
						<span class="info-label">
							{puzzleKind === 'daily' ? '今日题目' : puzzleKind === 'custom' ? '自定义关卡' : '随机种子'}
						</span>
						<span class="info-value font-mono">
							{puzzleKind === 'daily'
								? dateYmd || '--'
								: puzzleKind === 'custom'
									? shortLevelCode(safeLevelCodeForDisplay())
									: seed
										? shortSeed(seed)
										: '--'}
						</span>
						<span class="difficulty-chip" title="步数 / 提示次数">{moveCount} / {hintCount}</span>
						<span class="difficulty-chip" title="用时">{formatDuration(totalTimeMs)}</span>
						{#if difficulty}
							<span
								class="difficulty-chip"
								title={`难度分=${difficulty.difficulty_score}`}
							>
								难度 {difficulty.difficulty_score}
							</span>
						{:else}
							<span class="difficulty-chip difficulty-unknown" title="难度分未计算">难度 --</span>
						{/if}
					</div>

					<div class="game-actions">
						<button
							class="btn btn-primary"
							on:click={sharePuzzle}
							disabled={puzzleKind === 'custom' ? false : !seed}
							title="分享题目"
						>
							分享
						</button>
						<button class="btn" on:click={undo} disabled={!canUndo} title="撤销（Ctrl/Cmd+Z）">撤销</button>
						<button class="btn" on:click={redo} disabled={!canRedo} title="重做（Ctrl/Cmd+Shift+Z）">重做</button>
						<button class="btn" on:click={newRandomPuzzle} disabled={!engine}>
							随机
						</button>
						{#if puzzleKind !== 'daily'}
							<button class="btn btn-ghost" on:click={() => newDailyPuzzle({ updateUrl: true })} disabled={!engine}>
								回到今日
							</button>
						{/if}
					</div>
				</div>

				{#if shareToast}
					<div class="toast" transition:slide={{ axis: 'y' }}>{shareToast}</div>
				{/if}

				{#if tutorialOpen}
					<div class="modal-layer" transition:fade={{ duration: 140 }}>
						<button
							class="modal-backdrop"
							type="button"
							aria-label="关闭新手引导"
							on:click={closeTutorialSessionOnly}
						></button>
						<div
							class="modal"
							role="dialog"
							aria-modal="true"
							aria-label="新手引导"
							tabindex="-1"
							bind:this={tutorialDialogEl}
							on:keydown={onTutorialKeydown}
						>
							<div class="modal-title">新手引导</div>
							<div class="modal-sub">快速掌握：勾选、提示、撤销、标记、日历与编辑器</div>

							<div class="modal-body">
								<div class="tips">
									<div class="tip">
										<div class="tip-k">目标</div>
										<div class="tip-v">满足颜色规则，并形成任意一条 5 连线（Bingo）。</div>
									</div>
									<div class="tip">
										<div class="tip-k">勾选</div>
										<div class="tip-v">点击格子勾选/取消；黑格固定勾选不可点击。</div>
									</div>
									<div class="tip">
										<div class="tip-k">撤销/重做</div>
										<div class="tip-v">按钮或快捷键：Ctrl/Cmd+Z、Ctrl/Cmd+Shift+Z。</div>
									</div>
									<div class="tip">
										<div class="tip-k">标记</div>
										<div class="tip-v">右键/长按循环：无 → 排除(⊘) → 问号(?)；键盘可用 M。</div>
									</div>
									<div class="tip">
										<div class="tip-k">提示</div>
										<div class="tip-v">
											“安全提示”来自传播/反证；会高亮受影响格并定位对应规则。你可点击“应用”。
										</div>
									</div>
									<div class="tip">
										<div class="tip-k">键盘</div>
										<div class="tip-v">方向键移动焦点；Space/Enter 操作当前格。</div>
									</div>
									<div class="tip">
										<div class="tip-k">入口</div>
										<div class="tip-v">日历：补做往期；统计：趋势与成就；编辑器：自制并分享关卡。</div>
									</div>
								</div>

								<label class="checkbox">
									<input type="checkbox" bind:checked={tutorialDontAutoShow} />
									<span>以后不再自动弹出（仍可点右上角“引导”再次打开）</span>
								</label>
							</div>

							<div class="modal-actions">
								<button class="btn btn-primary" type="button" on:click={closeTutorial}>开始游戏</button>
								<button class="btn btn-ghost" type="button" on:click={closeTutorialSessionOnly}>
									仅关闭
								</button>
							</div>
						</div>
					</div>
				{/if}

				{#if shareManualVisible && shareUrlForManualCopy}
					<div class="share-manual" transition:slide>
						<input class="input" readonly value={shareUrlForManualCopy} on:focus={selectAll} on:click={selectAll} />
					</div>
				{/if}

				{#if urlSeedError}
					<div class="hint-banner">{urlSeedError}</div>
				{/if}
				{#if urlLevelError}
					<div class="hint-banner">{urlLevelError}</div>
				{/if}

                <!-- 棋盘容器 -->
				<div class="matrix-wrapper">
					<Matrix
						grid={grid}
						checkedMask={checkedMask}
						marks={marks}
						colorBlindMode={$colorBlindEnabled}
						highlightCells={hintExplainCells}
						highlightCellsSecondary={hintExplainDetailsOpen ? hintExplainSecondaryCells : []}
						cellOk={validate?.cell_ok ?? Array(25).fill(true)}
						hintIndex={hintIndex}
						hintAction={hintAction}
						onToggle={toggle}
						onMarkCycle={cycleMark}
						onHover={handleHover}
					/>
				</div>

                <!-- 状态指示条 -->
				{#if validate}
					<div class="status-bar">
                        <div class="status-item {validate.is_valid ? 'status-success' : 'status-warn'}">
                            <span class="status-dot"></span>
                            <span>约束：{validate.is_valid ? '满足' : '未满足'}</span>
                        </div>
                        <div class="status-item {validate.is_bingo ? 'status-success' : 'status-neutral'}">
                            <span class="status-dot"></span>
                            <span>目标：{validate.is_bingo ? '达成' : '进行中'}</span>
                        </div>
					</div>
				{/if}

				{#if dev}
					<details class="debug-panel">
						<summary>调试信息</summary>
						<div class="debug-content">
							<code>seed: {seed?.toString() ?? '—'}</code>
							<code>mask: {checkedMask >>> 0}</code>
							<code>difficulty: {difficulty ? `${difficulty.difficulty_score} (nodes=${difficulty.stats.node_visits})` : 'n/a'}</code>
						</div>
					</details>
				{/if}

				<!-- 规则面板：放在棋盘下方，便于对照理解 -->
				<div class="sidebar-card rules-panel rules-below">
					<div class="panel-header">
						<h2 class="panel-title">规则详情</h2>
						{#if !hoveredRule}
							<p class="panel-hint">悬停或点击格子定位规则</p>
						{/if}
					</div>

					<!-- 动态高亮区域：当“全部规则”展开时隐藏，避免重复呈现造成不协调 -->
					{#if !allRulesOpen}
						<div class="active-rule-section">
							{#if hoveredRule}
								<div class="section-label">当前关注</div>
								<div transition:slide={{ duration: 200 }}>
									<RuleCard rule={hoveredRule} color={ruleColorCss(hoveredRule.id)} highlighted />
								</div>
							{:else}
								<div class="empty-placeholder">
									<span>移动鼠标查看规则</span>
								</div>
							{/if}
						</div>
					{/if}

					{#if validate && activeCellIndex !== null && !validate.cell_ok[activeCellIndex] && validate.cell_messages[activeCellIndex]}
						<div class="cell-error-section">
							<div class="section-label">冲突原因</div>
							<div class="cell-error-box">
								<div class="cell-error-meta">
									格子 ({Math.floor(activeCellIndex / 5) + 1},{(activeCellIndex % 5) + 1})
								</div>
								<div class="cell-error-text">{validate.cell_messages[activeCellIndex]}</div>
							</div>
						</div>
					{/if}

					{#if goalRule}
						<div class="static-rule-section">
							<div class="section-label">通关目标</div>
							<RuleCard rule={goalRule} color={ruleColorCss(goalRule.id)} />
						</div>
					{/if}

					<details
						class="all-rules-details"
						bind:this={allRulesDetailsEl}
						on:toggle={(e) => (allRulesOpen = (e.currentTarget as HTMLDetailsElement).open)}
					>
						<summary>
							全部规则 <span class="badge-count">{allRules.length}</span>
						</summary>
						<div class="rules-grid">
							{#each allRules as r}
								<div class="rule-item" use:ruleRef={r.id}>
									<RuleCard
										rule={r}
										color={ruleColorCss(r.id)}
										highlighted={hoveredRuleId === r.id}
										highlightTone="soft"
									/>
								</div>
							{/each}
						</div>
					</details>
				</div>
			</main>

			<!-- 右侧：帮助/存档/提示 -->
			<aside class="card sidebar">
                <!-- 1. 帮助移到这里，并设为折叠，节省空间 -->
                <div class="sidebar-card help-section">
                    <details>
                        <summary class="help-summary">怎么玩？</summary>
                        <ul class="help-list">
                            <li><strong>点击非黑格：</strong>切换勾选状态。</li>
                            <li><strong>黑格：</strong>已锁定，必须勾选。</li>
                            <li><strong>红框/叹号：</strong>违反规则，原因会显示在棋盘下方“规则详情”里。</li>
                        </ul>
                    </details>
                </div>

				<!-- 1.5 提示：优先给出“安全一步”，卡住时再用“建议一步” -->
				<!-- 1.2 存档/历史 + 撤销/重做 -->
				<div class="sidebar-card progress-panel">
					<div class="panel-header">
						<h2 class="panel-title">存档</h2>
						<p class="panel-hint">自动保存当前关卡，支持撤销/重做与历史恢复。</p>
					</div>

					<div class="progress-current">
						<div class="progress-row">
							<div class="progress-title">当前</div>
							<span class="progress-chip {isSolved ? 'safe' : ''}">{isSolved ? '已通关' : '进行中'}</span>
						</div>
						{#if solvedAt}
							<div class="progress-subtext">通关时间：{formatTimestamp(solvedAt)}</div>
						{/if}
						<div class="progress-actions">
							<button class="btn" on:click={undo} disabled={!canUndo} title="撤销（Ctrl/Cmd+Z）">撤销</button>
							<button class="btn" on:click={redo} disabled={!canRedo} title="重做（Ctrl/Cmd+Shift+Z）">重做</button>
							<button class="btn btn-ghost" on:click={resetCurrentProgressWithConfirm} disabled={!currentProgressKey}>
								重置
							</button>
						</div>
					</div>

					<div class="progress-list">
						{#if progressEntries.length === 0}
							<div class="empty-placeholder">暂无存档</div>
						{:else}
							{#each progressEntries as e (e.key)}
								<div class="progress-item {e.key === currentProgressKey ? 'active' : ''}">
									<div class="progress-main">
										<div class="progress-title">{progressTitle(e)}</div>
										<div class="progress-sub">
											<span class="progress-chip">{progressBadge(e)}</span>
											<span class="progress-subtext">{formatTimestamp(e.updatedAt)}</span>
										</div>
									</div>
									<div class="progress-buttons">
										<button class="btn" on:click={() => openProgressEntry(e)} disabled={!engine}>打开</button>
										<button class="btn btn-ghost" on:click={() => removeProgressEntry(e.key)}>删除</button>
									</div>
								</div>
							{/each}
						{/if}
					</div>

					<div class="progress-footer">
						<button class="btn btn-ghost" on:click={clearAllProgressWithConfirm} disabled={progressEntries.length === 0}>
							清空全部
						</button>
					</div>
				</div>

				<div class="sidebar-card hint-section">
					<div class="panel-header hint-header">
						<h2 class="panel-title">提示</h2>
						<div class="hint-actions">
							<button class="btn btn-primary" on:click={requestHint} disabled={!engine || hintLoading} title="给我一步提示">
								{hintLoading ? '提示中…' : '给我提示'}
							</button>
							{#if hint?.move}
								<button class="btn" on:click={applyHintMove} disabled={hintLoading} title="应用该提示">
									应用
								</button>
							{/if}
							{#if hint}
								<button class="btn btn-ghost" on:click={clearHint} disabled={hintLoading} title="清除提示高亮">
									清除
								</button>
							{/if}
							<button class="btn btn-ghost" on:click={clearMarks} disabled={!hasMarks} title="清除标记（右键/长按循环）">
								清标记
							</button>
						</div>
						{#if !hint}
							<p class="panel-hint">优先给安全一步；若无强制结论，会给一个建议方向。</p>
						{/if}
					</div>

					{#if hint}
						<div class="hint-body">
							<div class="hint-message">{hint.message}</div>
							{#if hintExplain}
								<div class="hint-explain">{hintExplain}</div>
							{/if}
							{#if (hintExplainSecondaryCells.length > 0 || (hint.reason?.steps?.length ?? 0) > 0) && hint.move}
								<div class="hint-detail-row">
									<button class="btn btn-ghost" type="button" on:click={() => (hintExplainDetailsOpen = !hintExplainDetailsOpen)}>
										{hintExplainDetailsOpen ? '收起详细' : '详细解释'}
									</button>
									<span class="hint-detail-hint">展开后会显示推导链并追加高亮范围</span>
								</div>
							{/if}
							{#if hintExplainDetailsOpen && (hint.reason?.steps?.length ?? 0) > 0}
								<ol class="hint-steps">
									{#each hint.reason?.steps ?? [] as s}
										<li class="hint-step">
											<div class="hint-step-title">{s.title}</div>
										</li>
									{/each}
								</ol>
							{/if}
							{#if hint.move}
								<div class="hint-meta">
									<span class="hint-badge {hint.move.forced ? 'safe' : 'suggest'}">
										{hint.move.forced ? '安全提示' : '建议'}
									</span>
									<span class="hint-op">
										{hint.move.action === 'check' ? '勾选' : '取消勾选'} 格子
										({Math.floor(hint.move.cell / 5) + 1},{(hint.move.cell % 5) + 1})
									</span>
								</div>
							{/if}
						</div>
					{/if}
				</div>

			</aside>
		</div>
	{/if}
</div>

<style>
	.page-container {
		max-width: 1180px;
		margin: 0 auto;
	}

    /* 字体工具 */
    .font-mono { font-family: var(--mono); }

	/* Header */
	.header {
		display: flex;
		justify-content: space-between;
		align-items: flex-end;
		gap: 16px;
		margin-bottom: 18px;
	}

	.logo-area {
		display: grid;
		gap: 4px;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.github-link {
		padding: 8px 12px;
		display: inline-flex;
		align-items: center;
		color: var(--text);
	}

	.github-link .icon {
		width: 16px;
		height: 16px;
	}

	.logo {
		font-size: 1.9rem;
		font-weight: 900;
		letter-spacing: -0.06em;
		margin: 0;
		color: var(--text);
	}

	.subtitle {
		margin: 0;
		color: var(--muted);
		font-size: 0.95rem;
		font-weight: 550;
	}

	/* Main Layout */
	.main-layout {
		display: grid;
		gap: 18px;
		align-items: start;
	}

	@media (min-width: 900px) {
		.main-layout {
			grid-template-columns: minmax(0, 1fr) 360px;
		}
        .sidebar {
            position: sticky;
            top: 24px;
        }
	}

	/* Panels（容器外观由全局 .card 统一提供） */
	.game-area {
		padding: 16px 16px 18px;
	}

	.sidebar {
		padding: 16px;
	}

    /* --- Game Area Styles --- */

    /* Toolbar */
    .toolbar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 12px;
        padding-bottom: 14px;
        margin-bottom: 14px;
        border-bottom: 1px solid var(--border);
        flex-wrap: wrap;
    }

    .game-info {
        display: flex;
        align-items: baseline;
        gap: 10px;
        flex-wrap: wrap;
    }
    
    .info-label {
        font-size: 0.72rem;
        text-transform: uppercase;
        color: var(--muted-2);
        font-weight: 800;
        letter-spacing: 0.08em;
    }

    .info-value {
        font-weight: 650;
        font-size: 1rem;
        color: var(--text);
    }

	.difficulty-chip {
		display: inline-flex;
		align-items: center;
		margin-left: 10px;
		padding: 3px 10px;
		border-radius: 999px;
		font-size: 0.82rem;
		font-weight: 750;
		border: 1px solid var(--border);
		background: var(--bg-2);
		color: var(--muted);
	}

	.difficulty-unknown {
		opacity: 0.8;
	}

    .game-actions {
        display: flex;
        gap: 10px;
    }

    /* Matrix Container */
    .matrix-wrapper {
        display: flex;
        justify-content: center;
        background: var(--bg-2);
        padding: 18px;
        border-radius: var(--radius-xl);
        border: 1px solid var(--border);
        box-shadow: var(--inset-highlight), var(--inset-shadow);
    }
    
    @media (min-width: 900px) {
        .matrix-wrapper { justify-content: center; min-height: 480px; align-items: center; }
    }

    /* Status Bar */
    .status-bar {
        display: flex;
        gap: 10px;
        margin-top: 14px;
        justify-content: center;
        flex-wrap: wrap;
    }
    @media (min-width: 900px) { .status-bar { justify-content: flex-start; } }

    .status-item {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 12px;
        border-radius: 99px;
        font-size: 0.9rem;
        font-weight: 650;
        background: var(--bg-2);
        border: 1px solid var(--border);
        box-shadow: var(--inset-highlight);
    }

    .status-dot {
        width: 9px;
        height: 9px;
        border-radius: 50%;
        background: var(--muted-2);
    }

    .status-success {
        color: color-mix(in srgb, var(--success) 70%, var(--text));
        border-color: color-mix(in srgb, var(--success) 38%, var(--border));
        background: color-mix(in srgb, var(--success) 12%, var(--bg-2));
    }
    .status-success .status-dot { background: var(--success); }

    .status-warn {
        color: color-mix(in srgb, var(--danger) 70%, var(--text));
        border-color: color-mix(in srgb, var(--danger) 38%, var(--border));
        background: color-mix(in srgb, var(--danger) 12%, var(--bg-2));
    }
    .status-warn .status-dot { background: var(--danger); }

    .status-neutral { color: var(--text); }

    /* --- Sidebar Styles --- */

    .sidebar-card {
        background: transparent;
        border: 0;
        box-shadow: none;
        overflow: visible;
        margin-bottom: 16px;
    }

    .help-section {
        font-size: 0.92rem;
    }
    .help-summary {
        padding: 10px 12px;
        font-weight: 800;
        cursor: pointer;
        user-select: none;
        background: var(--bg-2);
        border: 1px solid var(--border);
        border-radius: var(--radius-lg);
        color: var(--text);
    }
    .help-summary:hover {
        border-color: var(--border-2);
    }
    .help-list {
        margin: 10px 0 0;
        padding: 10px 12px 0 26px;
        color: var(--muted);
        line-height: 1.55;
    }

	.progress-current {
		padding: 10px 12px;
		background: var(--bg-2);
		border: 1px solid var(--border);
		border-radius: var(--radius-lg);
	}

	.progress-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
	}

	.progress-title {
		font-weight: 900;
		color: var(--text);
		font-size: 0.92rem;
	}

	.progress-sub {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-top: 6px;
	}

	.progress-subtext {
		color: var(--muted);
		font-size: 0.82rem;
	}

	.progress-chip {
		font-size: 0.72rem;
		padding: 2px 8px;
		border-radius: 999px;
		font-weight: 850;
		letter-spacing: 0.02em;
		border: 1px solid var(--border);
		background: var(--panel);
		color: var(--muted);
		white-space: nowrap;
	}

	.progress-chip.safe {
		background: color-mix(in srgb, var(--success) 12%, var(--panel));
		color: color-mix(in srgb, var(--success) 70%, var(--text));
		border-color: color-mix(in srgb, var(--success) 35%, var(--border));
	}

	.progress-actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 10px;
	}

	.progress-list {
		margin-top: 12px;
		display: grid;
		gap: 10px;
	}

	.progress-item {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: 10px;
		padding: 10px 12px;
		border: 1px solid var(--border);
		border-radius: var(--radius-lg);
		background: var(--panel);
	}

	.progress-item.active {
		border-color: var(--border-2);
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--c-blue) 18%, transparent);
	}

	.progress-buttons {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		justify-content: flex-end;
	}

	.progress-footer {
		margin-top: 10px;
		display: flex;
		justify-content: flex-end;
	}

	.hint-header {
		display: grid;
		gap: 10px;
	}

	.hint-actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 8px;
	}

	.hint-body {
		margin-top: 10px;
		padding: 10px 12px;
		background: var(--bg-2);
		border: 1px solid var(--border);
		border-radius: var(--radius-lg);
		box-shadow: var(--inset-highlight);
	}

	.hint-message {
		font-size: 0.9rem;
		color: var(--text);
		line-height: 1.5;
	}

	.hint-explain {
		margin-top: 8px;
		font-size: 0.82rem;
		color: var(--muted);
		line-height: 1.5;
		white-space: pre-line;
	}

	.hint-detail-row {
		margin-top: 10px;
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
	}

	.hint-detail-hint {
		font-size: 0.82rem;
		color: var(--muted);
	}

	.hint-steps {
		margin: 10px 0 0;
		padding-left: 18px;
		color: var(--text);
	}

	.hint-step {
		margin: 6px 0;
	}

	.hint-step-title {
		font-size: 0.86rem;
		line-height: 1.45;
	}

	.hint-meta {
		margin-top: 8px;
		display: flex;
		align-items: center;
		gap: 10px;
		color: var(--muted);
	}

	.hint-badge {
		font-size: 0.7rem;
		padding: 2px 8px;
		border-radius: 999px;
		font-weight: 800;
		letter-spacing: 0.02em;
		white-space: nowrap;
		border: 1px solid var(--border);
		background: var(--panel);
		color: var(--muted);
	}

	.hint-badge.safe {
		background: color-mix(in srgb, var(--success) 12%, var(--panel));
		color: color-mix(in srgb, var(--success) 70%, var(--text));
		border-color: color-mix(in srgb, var(--success) 35%, var(--border));
	}

	.hint-badge.suggest {
		background: color-mix(in srgb, var(--c-blue) 10%, var(--panel));
		color: color-mix(in srgb, var(--c-blue) 72%, var(--text));
		border-color: color-mix(in srgb, var(--c-blue) 32%, var(--border));
	}

	.hint-op {
		font-size: 0.85rem;
	}

    .rules-panel {
        padding: 0;
    }

	.rules-below {
		margin-top: 16px;
	}

    .panel-header {
        margin: 18px 0 14px;
        padding-top: 14px;
        border-top: 1px solid var(--border);
    }

    .panel-title {
        font-size: 1.08rem;
        font-weight: 850;
        margin: 0;
        color: var(--text);
    }

    .panel-hint {
        font-size: 0.85rem;
        color: var(--muted);
        margin: 6px 0 0 0;
    }

    .section-label {
        font-size: 0.72rem;
        text-transform: uppercase;
        color: var(--muted-2);
        font-weight: 900;
        letter-spacing: 0.08em;
        margin-bottom: 8px;
    }

    .active-rule-section {
        min-height: 80px; /* 避免高度跳动 */
        margin-bottom: 16px;
    }

	.cell-error-section {
		margin-bottom: 14px;
	}

	.cell-error-box {
		background: var(--danger-surface);
		border: 1px solid var(--danger-border);
		border-radius: var(--radius-md);
		padding: 10px 12px;
	}

	.cell-error-meta {
		font-size: 0.8rem;
		color: var(--danger-meta);
		margin-bottom: 6px;
	}

	.cell-error-text {
		color: var(--danger-text);
		font-size: 0.92rem;
		line-height: 1.45;
	}

    .empty-placeholder {
        display: grid;
        place-items: center;
        height: 60px;
        background: var(--bg-2);
        border-radius: var(--radius-md);
        border: 1px dashed var(--border-2);
        color: var(--muted);
        font-size: 0.9rem;
    }

    .static-rule-section { margin-bottom: 14px; }

    .all-rules-details summary {
        cursor: pointer;
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-size: 0.95rem;
        font-weight: 750;
        color: var(--text);
        padding: 10px 0;
        border-top: 1px solid var(--border);
    }

    .badge-count {
        background: var(--bg-2);
        border: 1px solid var(--border);
        color: var(--muted);
        font-size: 0.8rem;
        padding: 2px 8px;
        border-radius: 99px;
    }
    
    .rules-grid {
        display: grid;
        gap: 10px;
        padding-top: 10px;
    }

    /* Toast & Utils */
	.toast {
		background: var(--tooltip-bg);
		border: 1px solid var(--tooltip-border);
		color: var(--tooltip-text);
		padding: 10px 12px;
		border-radius: var(--radius-sm);
		text-align: center;
		font-size: 0.92rem;
		margin-bottom: 12px;
		box-shadow: var(--shadow-soft);
	}

	.modal-layer {
		position: fixed;
		inset: 0;
		display: grid;
		place-items: center;
		padding: 18px;
		z-index: 9999;
	}

	.modal-backdrop {
		position: absolute;
		inset: 0;
		background: color-mix(in srgb, #000 55%, transparent);
		border: 0;
		padding: 0;
		margin: 0;
		appearance: none;
		cursor: pointer;
	}

	.modal {
		position: relative;
		width: min(760px, 100%);
		background: var(--panel);
		border: 1px solid var(--border);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-soft);
		padding: 16px 16px;
	}

	.modal-title {
		font-weight: 950;
		font-size: 1.05rem;
		letter-spacing: -0.02em;
		margin-bottom: 4px;
	}

	.modal-sub {
		color: var(--muted);
		font-size: 0.9rem;
		margin-bottom: 12px;
	}

	.modal-body {
		background: var(--bg-2);
		border: 1px solid var(--border);
		border-radius: var(--radius-md);
		padding: 12px 12px;
	}

	.tips {
		display: grid;
		gap: 10px;
	}

	.tip {
		display: grid;
		gap: 4px;
		padding: 10px 10px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--panel-2);
	}

	.tip-k {
		font-weight: 900;
		font-size: 0.85rem;
		color: var(--text);
	}

	.tip-v {
		color: var(--muted);
		font-size: 0.86rem;
		line-height: 1.45;
	}

	.checkbox {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-top: 12px;
		color: var(--muted);
		font-size: 0.9rem;
	}

	.modal-actions {
		display: flex;
		gap: 10px;
		justify-content: flex-end;
		margin-top: 12px;
	}

	@media (max-width: 720px) {
		.modal {
			padding: 14px 14px;
		}
		.modal-body {
			padding: 10px 10px;
		}
		.modal-actions {
			justify-content: stretch;
		}
		.modal-actions > :global(.btn) {
			flex: 1;
		}
	}

    .share-manual {
        margin-bottom: 12px;
    }

    .hint-banner {
        background: var(--danger-surface);
        border: 1px solid var(--danger-border);
        color: var(--danger-text);
        padding: 10px 12px;
        border-radius: var(--radius-sm);
        margin-bottom: 12px;
    }

    .error-banner {
        background: var(--danger-surface);
        border: 1px solid var(--danger-border);
        color: var(--danger-text);
        padding: 12px 14px;
        border-radius: var(--radius-lg);
        text-align: center;
        margin-bottom: 16px;
    }

    .debug-panel { margin-top: 16px; opacity: 0.75; font-size: 0.85rem; }

    .debug-content {
        display: grid;
        gap: 6px;
        padding-top: 10px;
    }
</style>
