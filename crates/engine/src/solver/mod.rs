use crate::colors::Color;
use crate::masks::{Mask, CELL_COUNT, GRID_SIZE};

mod state;

use state::SolverState;

#[inline]
fn cell_id(row: usize, col: usize, size: usize) -> usize {
    row * size + col
}

#[derive(Clone, Debug)]
struct RuleSet {
    size: usize,

    black_cells: Vec<usize>,
    blue_cells: Vec<usize>,
    red_cells: Vec<usize>,
    green_cells: Vec<usize>,
    yellow_cells: Vec<usize>,
    purple_cells: Vec<usize>,
    orange_cells: Vec<usize>,
    cyan_cells: Vec<usize>,

    decision_order: Vec<usize>,

    neighbors8: Vec<Vec<usize>>,
    neighbors4: Vec<Vec<usize>>,

    diag_down_cells: Vec<Vec<usize>>,
    diag_up_cells: Vec<Vec<usize>>,
}

impl RuleSet {
    fn new(size: usize, colors: Vec<Color>) -> Self {
        assert!((1..=32).contains(&size), "size 必须在 1..=32，得到：{size}");
        assert_eq!(colors.len(), size * size, "colors 长度必须为 size*size");

        let cell_count = size * size;
        let diag_count = size * 2 - 1;

        let mut black_cells = Vec::new();
        let mut blue_cells = Vec::new();
        let mut red_cells = Vec::new();
        let mut green_cells = Vec::new();
        let mut yellow_cells = Vec::new();
        let mut purple_cells = Vec::new();
        let mut orange_cells = Vec::new();
        let mut cyan_cells = Vec::new();

        for id in 0..cell_count {
            match colors[id] {
                Color::Black => black_cells.push(id),
                Color::Blue => blue_cells.push(id),
                Color::Red => red_cells.push(id),
                Color::Green => green_cells.push(id),
                Color::Yellow => yellow_cells.push(id),
                Color::Purple => purple_cells.push(id),
                Color::Orange => orange_cells.push(id),
                Color::Cyan => cyan_cells.push(id),
                _ => {}
            }
        }

        let mut neighbors8 = vec![Vec::new(); cell_count];
        let mut neighbors4 = vec![Vec::new(); cell_count];

        for row in 0..size {
            for col in 0..size {
                let id = cell_id(row, col, size);

                for dr in -1i32..=1 {
                    for dc in -1i32..=1 {
                        if dr == 0 && dc == 0 {
                            continue;
                        }
                        let nr = row as i32 + dr;
                        let nc = col as i32 + dc;
                        if nr < 0 || nc < 0 || nr >= size as i32 || nc >= size as i32 {
                            continue;
                        }
                        neighbors8[id].push(cell_id(nr as usize, nc as usize, size));
                    }
                }

                if row > 0 {
                    neighbors4[id].push(cell_id(row - 1, col, size));
                }
                if row + 1 < size {
                    neighbors4[id].push(cell_id(row + 1, col, size));
                }
                if col > 0 {
                    neighbors4[id].push(cell_id(row, col - 1, size));
                }
                if col + 1 < size {
                    neighbors4[id].push(cell_id(row, col + 1, size));
                }
            }
        }

        let mut diag_down_cells = vec![Vec::new(); diag_count];
        let mut diag_up_cells = vec![Vec::new(); diag_count];
        for row in 0..size {
            for col in 0..size {
                let id = cell_id(row, col, size);
                diag_down_cells[row + (size - 1) - col].push(id);
                diag_up_cells[row + col].push(id);
            }
        }

        let mut decision_order: Vec<usize> = (0..cell_count)
            .filter(|&id| colors[id] != Color::Black)
            .collect();
        decision_order.sort_by_key(|&id| {
            let neighbor_degree = neighbors8[id].len() as i32;
            let color_weight = match colors[id] {
                Color::Blue => 100,
                Color::Green => 90,
                Color::Yellow => 80,
                Color::Cyan => 70,
                Color::Red => 60,
                Color::Purple => 50,
                Color::Orange => 50,
                _ => 10,
            };
            -(color_weight + neighbor_degree)
        });

        Self {
            size,
            black_cells,
            blue_cells,
            red_cells,
            green_cells,
            yellow_cells,
            purple_cells,
            orange_cells,
            cyan_cells,
            decision_order,
            neighbors8,
            neighbors4,
            diag_down_cells,
            diag_up_cells,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SolveStats {
    pub node_visits: u64,
    pub decision_points: u64,
    pub branch_attempts: u64,
    pub dead_ends: u64,
    pub solutions: u64,

    pub propagate_rounds: u64,

    pub assignments_initial: u64,
    pub assignments_guess: u64,
    pub assignments_propagated: u64,

    pub max_depth: u32,

    // 推理入口隐蔽度相关
    pub rule_trigger_counts: std::collections::HashMap<RuleType, u64>,
    pub first_trigger_counts: std::collections::HashMap<RuleType, u64>,
    
    // 回溯距离相关
    pub decision_point_depths: Vec<u32>,
    pub backtrack_distances: Vec<u32>,
    pub avg_backtrack_distance: f64,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct HumanDifficultyAnalysis {
    pub solved: bool,
    pub exhausted_budget: bool,

    pub variable_cells: u32,
    pub initial_unknown_after_logic: u32,

    /// 当前已确定“不勾选”的格子会排除对应的 5 连线段；该指标用于衡量“Bingo 目标”的约束强度。
    pub bingo_segments_total: u32,
    pub bingo_segments_possible: u32,
    pub bingo_segments_guaranteed: u32,

    /// 主路径上的逻辑传播开销（不包含“反证分支”的开销）。
    pub logic_propagate_rounds: u64,
    pub logic_assignments_propagated: u64,
    /// 发生过“推出至少 1 个强制结论”的逻辑爆发次数（用于刻画“推得动 -> 卡住 -> 再推得动”的断档感）。
    pub logic_bursts: u32,
    pub max_logic_burst_size: u32,
    pub logic_rule_trigger_counts: std::collections::HashMap<RuleType, u64>,
    pub logic_first_trigger_counts: std::collections::HashMap<RuleType, u64>,

    /// 反证（假设某格取值）走到矛盾所消耗的传播开销。
    pub contradiction_propagate_rounds: u64,
    pub contradiction_assignments_propagated: u64,
    /// 每次能做反证推出强制结论时，“可用入口”有多稀缺（越稀缺，人越容易卡住）。
    pub contradiction_entry_total_assumptions: u32,
    pub contradiction_entry_candidate_assumptions: u32,
    pub contradiction_entry_scarcity_sum: f64,
    pub contradiction_entry_scarcity_max: f64,

    /// 通过“假设 -> 推理 -> 矛盾”得到的强制步数（人类常见的反证法）。
    pub forced_by_contradiction: u32,

    /// “真猜”的次数：在无法继续推出强制时，需要二选一推进。
    pub guesses: u32,
    pub max_guess_depth: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AssignReason {
    Initial,
    Guess,
    Propagate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum RuleType {
    Green,
    Yellow,
    Red,
    Blue,
    Purple,
    Orange,
    Cyan,
    FiveInRow,
}

trait SolveObserver {
    fn on_node(&mut self, _depth: u32) {}
    fn on_decision_point(&mut self, _depth: u32) {}
    fn on_branch_attempt(&mut self) {}
    fn on_dead_end(&mut self, _depth: u32) {}
    fn on_solution(&mut self) {}
    fn on_propagate_round(&mut self) {}
    fn on_assignment(&mut self, _reason: AssignReason) {}
    fn on_rule_trigger(&mut self, _rule: RuleType, _is_first: bool) {}
    fn on_backtrack(&mut self, _from_depth: u32, _to_depth: u32) {}
}

impl SolveObserver for () {}

#[derive(Clone, Debug, Default)]
struct PropagationObserver {
    propagate_rounds: u64,
    assignments_propagated: u64,
    rule_trigger_counts: std::collections::HashMap<RuleType, u64>,
    first_trigger_counts: std::collections::HashMap<RuleType, u64>,
}

impl PropagationObserver {
    fn merge_into_analysis(&self, analysis: &mut HumanDifficultyAnalysis) {
        analysis.logic_propagate_rounds = analysis.logic_propagate_rounds.saturating_add(self.propagate_rounds);
        analysis.logic_assignments_propagated =
            analysis.logic_assignments_propagated.saturating_add(self.assignments_propagated);
        for (&rule, &count) in &self.rule_trigger_counts {
            *analysis.logic_rule_trigger_counts.entry(rule).or_insert(0) += count;
        }
        for (&rule, &count) in &self.first_trigger_counts {
            *analysis.logic_first_trigger_counts.entry(rule).or_insert(0) += count;
        }
    }

    fn merge_into_contradiction(&self, analysis: &mut HumanDifficultyAnalysis) {
        analysis.contradiction_propagate_rounds =
            analysis.contradiction_propagate_rounds.saturating_add(self.propagate_rounds);
        analysis.contradiction_assignments_propagated = analysis
            .contradiction_assignments_propagated
            .saturating_add(self.assignments_propagated);
    }
}

impl SolveObserver for PropagationObserver {
    fn on_propagate_round(&mut self) {
        self.propagate_rounds += 1;
    }

    fn on_assignment(&mut self, reason: AssignReason) {
        if matches!(reason, AssignReason::Propagate) {
            self.assignments_propagated += 1;
        }
    }

    fn on_rule_trigger(&mut self, rule: RuleType, is_first: bool) {
        *self.rule_trigger_counts.entry(rule).or_insert(0) += 1;
        if is_first {
            *self.first_trigger_counts.entry(rule).or_insert(0) += 1;
        }
    }
}

impl SolveObserver for SolveStats {
    fn on_node(&mut self, depth: u32) {
        self.node_visits += 1;
        self.max_depth = self.max_depth.max(depth);
    }

    fn on_decision_point(&mut self, depth: u32) {
        self.decision_points += 1;
        self.decision_point_depths.push(depth);
    }

    fn on_branch_attempt(&mut self) {
        self.branch_attempts += 1;
    }

    fn on_dead_end(&mut self, depth: u32) {
        self.dead_ends += 1;
        // 计算回溯距离：当前深度 - 上一个决策点的深度
        if let Some(&last_dp_depth) = self.decision_point_depths.last() {
            let distance = depth.saturating_sub(last_dp_depth);
            self.backtrack_distances.push(distance);
            // 更新平均回溯距离
            let total_distances: u32 = self.backtrack_distances.iter().sum();
            self.avg_backtrack_distance = total_distances as f64 / self.backtrack_distances.len() as f64;
        }
    }

    fn on_solution(&mut self) {
        self.solutions += 1;
    }

    fn on_propagate_round(&mut self) {
        self.propagate_rounds += 1;
    }

    fn on_assignment(&mut self, reason: AssignReason) {
        match reason {
            AssignReason::Initial => self.assignments_initial += 1,
            AssignReason::Guess => self.assignments_guess += 1,
            AssignReason::Propagate => self.assignments_propagated += 1,
        }
    }

    fn on_rule_trigger(&mut self, rule: RuleType, is_first: bool) {
        // 更新规则触发计数
        *self.rule_trigger_counts.entry(rule).or_insert(0) += 1;
        // 如果是本轮第一个触发的规则，更新第一个触发规则计数
        if is_first {
            *self.first_trigger_counts.entry(rule).or_insert(0) += 1;
        }
    }

    fn on_backtrack(&mut self, from_depth: u32, to_depth: u32) {
        let distance = from_depth.saturating_sub(to_depth);
        self.backtrack_distances.push(distance);
        // 更新平均回溯距离
        let total_distances: u32 = self.backtrack_distances.iter().sum();
        self.avg_backtrack_distance = total_distances as f64 / self.backtrack_distances.len() as f64;
    }
}

fn try_set_checked<O: SolveObserver>(
    state: &mut SolverState,
    row: usize,
    col: usize,
    reason: AssignReason,
    obs: &mut O,
) -> bool {
    match state.set_checked(row, col) {
        Ok(true) => {
            obs.on_assignment(reason);
            true
        }
        Ok(false) => true,
        Err(()) => false,
    }
}

fn try_set_unchecked<O: SolveObserver>(
    state: &mut SolverState,
    row: usize,
    col: usize,
    reason: AssignReason,
    obs: &mut O,
) -> bool {
    match state.set_unchecked(row, col) {
        Ok(true) => {
            obs.on_assignment(reason);
            true
        }
        Ok(false) => true,
        Err(()) => false,
    }
}

fn try_set_checked_id<O: SolveObserver>(
    state: &mut SolverState,
    id: usize,
    reason: AssignReason,
    obs: &mut O,
) -> bool {
    match state.set_checked_id(id) {
        Ok(true) => {
            obs.on_assignment(reason);
            true
        }
        Ok(false) => true,
        Err(()) => false,
    }
}

fn try_set_unchecked_id<O: SolveObserver>(
    state: &mut SolverState,
    id: usize,
    reason: AssignReason,
    obs: &mut O,
) -> bool {
    match state.set_unchecked_id(id) {
        Ok(true) => {
            obs.on_assignment(reason);
            true
        }
        Ok(false) => true,
        Err(()) => false,
    }
}

/// 求解器（约束传播 + 猜测回溯）。
///
/// 说明：
/// - 黑格（Color::Black）强制勾选，不作为变量参与决策；
/// - 其余格子为变量：可勾选或不勾选；
/// - 解的判定：满足所有颜色规则且至少形成一条“五连线”（连续 5 个勾选，四个方向）。
pub struct Solver {
    rules: RuleSet,
}

impl Solver {
    pub fn new(colors: [Color; CELL_COUNT]) -> Self {
        Self {
            rules: RuleSet::new(GRID_SIZE, colors.to_vec()),
        }
    }

    /// 求解并返回最多 `limit` 个解（limit=0 视为不限制）。
    pub fn solve_masks_limit(&self, limit: usize) -> Vec<Mask> {
        let mut state = SolverState::new(self.rules.size);
        for &id in &self.rules.black_cells {
            // 黑格固定勾选，如果冲突交给传播阶段判定即可。
            let _ = state.set_checked_id(id);
        }

        let mut out = Vec::new();
        let mut obs = ();
        self.search(state, limit, &mut out, 0, &mut obs);
        out
    }

    pub(crate) fn solve_masks_limit_with_stats(
        &self,
        limit: usize,
        stats: &mut SolveStats,
    ) -> Vec<Mask> {
        let mut state = SolverState::new(self.rules.size);
        for &id in &self.rules.black_cells {
            if !try_set_checked_id(&mut state, id, AssignReason::Initial, stats) {
                return Vec::new();
            }
        }

        let mut out = Vec::new();
        self.search(state, limit, &mut out, 0, stats);
        out
    }

    pub(crate) fn analyze_human_difficulty(&self) -> HumanDifficultyAnalysis {
        const BUDGET: u32 = 50_000;

        let mut state = SolverState::new(self.rules.size);
        for &id in &self.rules.black_cells {
            if state.set_checked_id(id).is_err() {
                return HumanDifficultyAnalysis {
                    solved: false,
                    exhausted_budget: false,
                    variable_cells: self.rules.decision_order.len() as u32,
                    ..HumanDifficultyAnalysis::default()
                };
            }
        }

        let mut analysis = HumanDifficultyAnalysis::default();
        analysis.variable_cells = self.rules.decision_order.len() as u32;

        let mut budget = BUDGET;
        if self
            .propagate_logic_with_budget(&mut state, &mut budget, &mut analysis)
            .is_none()
        {
            analysis.solved = false;
            analysis.exhausted_budget = budget == 0;
            let (total, possible, guaranteed) = bingo_segment_stats(&state);
            analysis.bingo_segments_total = total;
            analysis.bingo_segments_possible = possible;
            analysis.bingo_segments_guaranteed = guaranteed;
            return analysis;
        }
        analysis.initial_unknown_after_logic = self.count_unknown_decision_cells(&state);

        // 只做“人类常用”的逻辑阶段：传播 + 反证推出强制。
        loop {
            let Some((cell, forced_checked, contradiction_obs, (unknown_cells, candidate_assumptions))) =
                self.find_forced_by_contradiction(&state, &mut budget)
            else {
                break;
            };

            analysis.forced_by_contradiction += 1;
            contradiction_obs.merge_into_contradiction(&mut analysis);

            // “断档”稀缺度：可用入口越少，人类越容易卡住。
            // 用 log2(total/candidates) 近似“需要试多少次/观察多久”。
            let total_assumptions = unknown_cells.saturating_mul(2);
            analysis.contradiction_entry_total_assumptions = analysis
                .contradiction_entry_total_assumptions
                .saturating_add(total_assumptions);
            analysis.contradiction_entry_candidate_assumptions = analysis
                .contradiction_entry_candidate_assumptions
                .saturating_add(candidate_assumptions);
            if total_assumptions > 0 && candidate_assumptions > 0 {
                let scarcity = (total_assumptions as f64 / candidate_assumptions as f64).log2();
                analysis.contradiction_entry_scarcity_sum += scarcity;
                analysis.contradiction_entry_scarcity_max =
                    analysis.contradiction_entry_scarcity_max.max(scarcity);
            }

            let assign_res = if forced_checked {
                state.set_checked_id(cell)
            } else {
                state.set_unchecked_id(cell)
            };
            if assign_res.is_err() {
                break;
            }

            if self
                .propagate_logic_with_budget(&mut state, &mut budget, &mut analysis)
                .is_none()
            {
                break;
            }
        }

        analysis.solved = state.is_fully_decided();
        analysis.exhausted_budget = budget == 0;

        let (total, possible, guaranteed) = bingo_segment_stats(&state);
        analysis.bingo_segments_total = total;
        analysis.bingo_segments_possible = possible;
        analysis.bingo_segments_guaranteed = guaranteed;

        analysis
    }

    fn count_unknown_decision_cells(&self, state: &SolverState) -> u32 {
        self.rules
            .decision_order
            .iter()
            .filter(|&&id| state.is_unknown_id(id))
            .count() as u32
    }

    fn propagate_logic_with_budget(
        &self,
        state: &mut SolverState,
        budget: &mut u32,
        analysis: &mut HumanDifficultyAnalysis,
    ) -> Option<()> {
        if *budget == 0 {
            return None;
        }
        *budget -= 1;

        let mut obs = PropagationObserver::default();
        if !self.propagate_to_fixpoint(state, &mut obs) {
            return None;
        }

        let burst_size = obs.assignments_propagated.min(u32::MAX as u64) as u32;
        if burst_size > 0 {
            analysis.logic_bursts = analysis.logic_bursts.saturating_add(1);
            analysis.max_logic_burst_size = analysis.max_logic_burst_size.max(burst_size);
        }
        obs.merge_into_analysis(analysis);
        Some(())
    }

    fn find_forced_by_contradiction(
        &self,
        state: &SolverState,
        budget: &mut u32,
    ) -> Option<(usize, bool, PropagationObserver, (u32, u32))> {
        // 统计“断档”稀缺度：在当前状态下，做一次“单步反证”共有多少可用入口？
        // - total_assumptions = 未知格子数 * 2
        // - candidate_assumptions = 假设后能直接导出矛盾的入口数
        //   （每个未知格子最多只有 1 个方向会矛盾，否则就是无解）
        let mut unknown_cells = 0u32;
        let mut candidate_assumptions = 0u32;

        let mut first_found: Option<(usize, bool, PropagationObserver)> = None;
        for &cell in &self.rules.decision_order {
            if !state.is_unknown_id(cell) {
                continue;
            }
            unknown_cells = unknown_cells.saturating_add(1);

            // 假设“不勾选” -> 若矛盾，则强制“勾选”。
            if let Some(contradiction) = self.contradiction_proof(state, cell, false, budget) {
                candidate_assumptions = candidate_assumptions.saturating_add(1);
                if first_found.is_none() {
                    first_found = Some((cell, true, contradiction));
                }
            }
            // 假设“勾选” -> 若矛盾，则强制“不勾选”。
            if let Some(contradiction) = self.contradiction_proof(state, cell, true, budget) {
                candidate_assumptions = candidate_assumptions.saturating_add(1);
                if first_found.is_none() {
                    first_found = Some((cell, false, contradiction));
                }
            }
        }

        first_found.map(|(cell, forced_checked, obs)| {
            (cell, forced_checked, obs, (unknown_cells, candidate_assumptions))
        })
    }

    fn contradiction_proof(
        &self,
        state: &SolverState,
        cell: usize,
        assume_checked: bool,
        budget: &mut u32,
    ) -> Option<PropagationObserver> {
        let mut fork = state.clone();
        let assign_res = if assume_checked {
            fork.set_checked_id(cell)
        } else {
            fork.set_unchecked_id(cell)
        };

        // 立即矛盾：不需要传播即可判定。
        if assign_res.is_err() {
            return Some(PropagationObserver::default());
        }

        if *budget == 0 {
            return None;
        }
        *budget -= 1;

        let mut obs = PropagationObserver::default();
        let ok = self.propagate_to_fixpoint(&mut fork, &mut obs);
        if ok { None } else { Some(obs) }
    }

    fn search(
        &self,
        state: SolverState,
        limit: usize,
        out: &mut Vec<Mask>,
        depth: u32,
        obs: &mut impl SolveObserver,
    ) {
        if limit != 0 && out.len() >= limit {
            return;
        }

        let mut state = state;
        obs.on_node(depth);
        if !self.propagate_to_fixpoint(&mut state, obs) {
            obs.on_dead_end(depth);
            return;
        }

        if state.is_fully_decided() {
            obs.on_solution();
            out.push(state.to_row_major_u32_mask());
            return;
        }

        let Some(cell) = self.find_next_unknown_cell(&state) else {
            return;
        };
        obs.on_decision_point(depth);

        // 分支 1：先尝试“不勾选”（对很多规则更容易快速剪枝）
        {
            let mut fork = state.clone();
            obs.on_branch_attempt();
            if try_set_unchecked_id(&mut fork, cell, AssignReason::Guess, obs) {
                self.search(fork, limit, out, depth + 1, obs);
            } else {
                obs.on_dead_end(depth + 1);
            }
        }

        if limit != 0 && out.len() >= limit {
            return;
        }

        // 分支 2：尝试“勾选”
        {
            let mut fork = state;
            obs.on_branch_attempt();
            if try_set_checked_id(&mut fork, cell, AssignReason::Guess, obs) {
                self.search(fork, limit, out, depth + 1, obs);
            } else {
                obs.on_dead_end(depth + 1);
            }
        }
    }

    fn find_next_unknown_cell(&self, state: &SolverState) -> Option<usize> {
        for &id in &self.rules.decision_order {
            if state.is_unknown_id(id) {
                return Some(id);
            }
        }
        None
    }

    fn propagate_to_fixpoint(&self, state: &mut SolverState, obs: &mut impl SolveObserver) -> bool {
        loop {
            let old = state.hash64();
            obs.on_propagate_round();

            let mut any_rule_triggered = false;

            // 传播规则并记录第一个触发的规则
            if !self.propagate_green(state, obs) {
                return false;
            }
            let green_triggered = state.hash64() != old;
            if green_triggered {
                obs.on_rule_trigger(RuleType::Green, !any_rule_triggered);
                any_rule_triggered = true;
            }

            let green_hash = state.hash64();
            if !self.propagate_yellow(state, obs) {
                return false;
            }
            let yellow_triggered = state.hash64() != green_hash;
            if yellow_triggered {
                obs.on_rule_trigger(RuleType::Yellow, !any_rule_triggered);
                any_rule_triggered = true;
            }

            let yellow_hash = state.hash64();
            if !self.propagate_blue(state, obs) {
                return false;
            }
            let blue_triggered = state.hash64() != yellow_hash;
            if blue_triggered {
                obs.on_rule_trigger(RuleType::Blue, !any_rule_triggered);
                any_rule_triggered = true;
            }

            let blue_hash = state.hash64();
            if !self.propagate_red(state, obs) {
                return false;
            }
            let red_triggered = state.hash64() != blue_hash;
            if red_triggered {
                obs.on_rule_trigger(RuleType::Red, !any_rule_triggered);
                any_rule_triggered = true;
            }

            let red_hash = state.hash64();
            if !self.propagate_purple(state, obs) {
                return false;
            }
            let purple_triggered = state.hash64() != red_hash;
            if purple_triggered {
                obs.on_rule_trigger(RuleType::Purple, !any_rule_triggered);
                any_rule_triggered = true;
            }

            let purple_hash = state.hash64();
            if !self.propagate_orange(state, obs) {
                return false;
            }
            let orange_triggered = state.hash64() != purple_hash;
            if orange_triggered {
                obs.on_rule_trigger(RuleType::Orange, !any_rule_triggered);
                any_rule_triggered = true;
            }

            let orange_hash = state.hash64();
            if !self.propagate_cyan(state, obs) {
                return false;
            }
            let cyan_triggered = state.hash64() != orange_hash;
            if cyan_triggered {
                obs.on_rule_trigger(RuleType::Cyan, !any_rule_triggered);
                any_rule_triggered = true;
            }

            let cyan_hash = state.hash64();
            if !self.propagate_five_in_a_row_possible(state) {
                return false;
            }
            let five_in_row_triggered = state.hash64() != cyan_hash;
            if five_in_row_triggered {
                obs.on_rule_trigger(RuleType::FiveInRow, !any_rule_triggered);
                any_rule_triggered = true;
            }

            if state.hash64() == old {
                break;
            }
        }
        true
    }

    fn propagate_green(&self, state: &mut SolverState, obs: &mut impl SolveObserver) -> bool {
        for &id in &self.rules.green_cells {
            let row = id / self.rules.size;
            let col = id % self.rules.size;
            let (r_min, r_max) = state.row_min_max(row);
            let (c_min, c_max) = state.col_min_max(col);

            if r_max < c_min || c_max < r_min {
                return false;
            }

            // 情况 A：Row 的 Max == Col 的 Min
            if r_max == c_min {
                if !fill_row_unknowns_as(state, row, true, obs) {
                    return false;
                }
                if !fill_col_unknowns_as(state, col, false, obs) {
                    return false;
                }
            }

            // 情况 B：Col 的 Max == Row 的 Min
            if c_max == r_min {
                if !fill_col_unknowns_as(state, col, true, obs) {
                    return false;
                }
                if !fill_row_unknowns_as(state, row, false, obs) {
                    return false;
                }
            }
        }
        true
    }

    fn propagate_yellow(&self, state: &mut SolverState, obs: &mut impl SolveObserver) -> bool {
        for &id in &self.rules.yellow_cells {
            let row = id / self.rules.size;
            let col = id % self.rules.size;

            let down_id = row + (self.rules.size - 1) - col;
            let up_id = row + col;

            let (d_min, d_max) = state.diag_down_min_max(down_id);
            let (u_min, u_max) = state.diag_up_min_max(up_id);

            if d_max < u_min || u_max < d_min {
                return false;
            }

            // 对角线推导与绿格一致（范围卡边界）
            if d_max == u_min {
                if !fill_cells_unknowns_as(state, &self.rules.diag_down_cells[down_id], true, obs) {
                    return false;
                }
                if !fill_cells_unknowns_as(state, &self.rules.diag_up_cells[up_id], false, obs) {
                    return false;
                }
            }
            if u_max == d_min {
                if !fill_cells_unknowns_as(state, &self.rules.diag_up_cells[up_id], true, obs) {
                    return false;
                }
                if !fill_cells_unknowns_as(state, &self.rules.diag_down_cells[down_id], false, obs)
                {
                    return false;
                }
            }
        }
        true
    }

    fn propagate_red(&self, state: &mut SolverState, obs: &mut impl SolveObserver) -> bool {
        for &id in &self.rules.red_cells {
            let mut checked = 0usize;
            let mut last_unknown: Option<usize> = None;
            let mut unknown_count = 0usize;

            for &n in &self.rules.neighbors8[id] {
                if state.is_checked_id(n) {
                    checked += 1;
                } else if state.is_unknown_id(n) {
                    unknown_count += 1;
                    last_unknown = Some(n);
                }
            }

            if checked > 0 {
                continue;
            }
            if unknown_count == 0 {
                return false;
            }
            if unknown_count == 1 {
                if !try_set_checked_id(
                    state,
                    last_unknown.expect("unknown_count==1"),
                    AssignReason::Propagate,
                    obs,
                ) {
                    return false;
                }
            }
        }
        true
    }

    fn propagate_blue(&self, state: &mut SolverState, obs: &mut impl SolveObserver) -> bool {
        for &id in &self.rules.blue_cells {
            let mut checked = 0usize;
            let mut unknowns = Vec::new();
            for &n in &self.rules.neighbors8[id] {
                if state.is_checked_id(n) {
                    checked += 1;
                } else if state.is_unknown_id(n) {
                    unknowns.push(n);
                }
            }

            if checked > 2 {
                return false;
            }

            // 已经 2 个勾选 -> 剩余未知邻居全部必须不勾选
            if checked == 2 {
                for n in unknowns {
                    if !try_set_unchecked_id(state, n, AssignReason::Propagate, obs) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn propagate_purple(&self, state: &mut SolverState, obs: &mut impl SolveObserver) -> bool {
        for &id in &self.rules.purple_cells {
            let mut checked = 0usize;
            let mut last_unknown: Option<usize> = None;
            let mut unknown_count = 0usize;

            for &n in &self.rules.neighbors8[id] {
                if state.is_checked_id(n) {
                    checked += 1;
                } else if state.is_unknown_id(n) {
                    unknown_count += 1;
                    last_unknown = Some(n);
                }
            }

            if unknown_count == 0 {
                if checked % 2 != 1 {
                    return false;
                }
                continue;
            }

            // 只剩 1 个未知时，可强制推导奇偶
            if unknown_count == 1 {
                let target = last_unknown.expect("unknown_count==1");
                let should_check = checked % 2 == 0;
                let ok = if should_check {
                    try_set_checked_id(state, target, AssignReason::Propagate, obs)
                } else {
                    try_set_unchecked_id(state, target, AssignReason::Propagate, obs)
                };
                if !ok {
                    return false;
                }
            }
        }
        true
    }

    fn propagate_orange(&self, state: &mut SolverState, obs: &mut impl SolveObserver) -> bool {
        for &id in &self.rules.orange_cells {
            let mut checked = 0usize;
            let mut last_unknown: Option<usize> = None;
            let mut unknown_count = 0usize;

            for &n in &self.rules.neighbors8[id] {
                if state.is_checked_id(n) {
                    checked += 1;
                } else if state.is_unknown_id(n) {
                    unknown_count += 1;
                    last_unknown = Some(n);
                }
            }

            if unknown_count == 0 {
                if checked % 2 != 0 {
                    return false;
                }
                continue;
            }

            // 只剩 1 个未知时，可强制推导奇偶
            if unknown_count == 1 {
                let target = last_unknown.expect("unknown_count==1");
                let should_check = checked % 2 == 1;
                let ok = if should_check {
                    try_set_checked_id(state, target, AssignReason::Propagate, obs)
                } else {
                    try_set_unchecked_id(state, target, AssignReason::Propagate, obs)
                };
                if !ok {
                    return false;
                }
            }
        }
        true
    }

    fn propagate_cyan(&self, state: &mut SolverState, obs: &mut impl SolveObserver) -> bool {
        for &id in &self.rules.cyan_cells {
            let (row, col) = (id / self.rules.size, id % self.rules.size);

            // 未勾选：规则不生效
            if state.is_unchecked(row, col) {
                continue;
            }

            // 已勾选：四邻至少 1 个勾选（类似红格，但邻域为 4）
            if state.is_checked(row, col) {
                let mut checked = 0usize;
                let mut last_unknown: Option<usize> = None;
                let mut unknown_count = 0usize;

                for &n in &self.rules.neighbors4[id] {
                    if state.is_checked_id(n) {
                        checked += 1;
                    } else if state.is_unknown_id(n) {
                        unknown_count += 1;
                        last_unknown = Some(n);
                    }
                }

                if checked > 0 {
                    continue;
                }
                if unknown_count == 0 {
                    return false;
                }
                if unknown_count == 1 {
                    if !try_set_checked_id(
                        state,
                        last_unknown.expect("unknown_count==1"),
                        AssignReason::Propagate,
                        obs,
                    ) {
                        return false;
                    }
                }
                continue;
            }

            // 未知：如果四邻全部“不勾选”，则该青格不能勾选（否则必矛盾）-> 强制不勾选
            let mut all_neighbors_unchecked = true;
            for &n in &self.rules.neighbors4[id] {
                if !state.is_unchecked_id(n) {
                    all_neighbors_unchecked = false;
                    break;
                }
            }
            if all_neighbors_unchecked {
                if !try_set_unchecked(state, row, col, AssignReason::Propagate, obs) {
                    return false;
                }
            }
        }
        true
    }

    fn propagate_five_in_a_row_possible(&self, state: &SolverState) -> bool {
        five_in_a_row_possible(state)
    }
}

fn fill_row_unknowns_as(
    state: &mut SolverState,
    row: usize,
    is_checked: bool,
    obs: &mut impl SolveObserver,
) -> bool {
    let mut mask = state.unknown_cols_mask_in_row(row);
    while mask != 0 {
        let col = mask.trailing_zeros() as usize;
        mask &= mask - 1;

        let ok = if is_checked {
            try_set_checked(state, row, col, AssignReason::Propagate, obs)
        } else {
            try_set_unchecked(state, row, col, AssignReason::Propagate, obs)
        };
        if !ok {
            return false;
        }
    }
    true
}

fn fill_col_unknowns_as(
    state: &mut SolverState,
    col: usize,
    is_checked: bool,
    obs: &mut impl SolveObserver,
) -> bool {
    let mut mask = state.unknown_rows_mask_in_col(col);
    while mask != 0 {
        let row = mask.trailing_zeros() as usize;
        mask &= mask - 1;

        let ok = if is_checked {
            try_set_checked(state, row, col, AssignReason::Propagate, obs)
        } else {
            try_set_unchecked(state, row, col, AssignReason::Propagate, obs)
        };
        if !ok {
            return false;
        }
    }
    true
}

fn fill_cells_unknowns_as(
    state: &mut SolverState,
    cell_ids: &[usize],
    is_checked: bool,
    obs: &mut impl SolveObserver,
) -> bool {
    for &id in cell_ids {
        if !state.is_unknown_id(id) {
            continue;
        }
        let ok = if is_checked {
            try_set_checked_id(state, id, AssignReason::Propagate, obs)
        } else {
            try_set_unchecked_id(state, id, AssignReason::Propagate, obs)
        };
        if !ok {
            return false;
        }
    }
    true
}

/// 五连线可行性剪枝：只要存在“长度为 5 的线段”不包含任何“已确定不勾选”的格子，就仍有希望达成目标。
fn five_in_a_row_possible(state: &SolverState) -> bool {
    let size = state.size();
    let len = 5usize;
    if size < len {
        return false;
    }

    // 横向
    for row in 0..size {
        for start_col in 0..=size - len {
            let mut ok = true;
            for dc in 0..len {
                if state.is_unchecked(row, start_col + dc) {
                    ok = false;
                    break;
                }
            }
            if ok {
                return true;
            }
        }
    }

    // 纵向
    for col in 0..size {
        for start_row in 0..=size - len {
            let mut ok = true;
            for dr in 0..len {
                if state.is_unchecked(start_row + dr, col) {
                    ok = false;
                    break;
                }
            }
            if ok {
                return true;
            }
        }
    }

    // 主对角线方向（\）
    for start_row in 0..=size - len {
        for start_col in 0..=size - len {
            let mut ok = true;
            for d in 0..len {
                if state.is_unchecked(start_row + d, start_col + d) {
                    ok = false;
                    break;
                }
            }
            if ok {
                return true;
            }
        }
    }

    // 副对角线方向（/）
    for start_row in 0..=size - len {
        for start_col in (len - 1)..size {
            let mut ok = true;
            for d in 0..len {
                if state.is_unchecked(start_row + d, start_col - d) {
                    ok = false;
                    break;
                }
            }
            if ok {
                return true;
            }
        }
    }

    false
}

fn bingo_segment_stats(state: &SolverState) -> (u32, u32, u32) {
    let size = state.size();
    let len = 5usize;
    if size < len {
        return (0, 0, 0);
    }

    let mut total = 0u32;
    let mut possible = 0u32;
    let mut guaranteed = 0u32;

    // 横向
    for row in 0..size {
        for start_col in 0..=size - len {
            total += 1;
            let mut ok = true;
            let mut all_checked = true;
            for dc in 0..len {
                let col = start_col + dc;
                if state.is_unchecked(row, col) {
                    ok = false;
                    break;
                }
                if !state.is_checked(row, col) {
                    all_checked = false;
                }
            }
            if ok {
                possible += 1;
                if all_checked {
                    guaranteed += 1;
                }
            }
        }
    }

    // 纵向
    for col in 0..size {
        for start_row in 0..=size - len {
            total += 1;
            let mut ok = true;
            let mut all_checked = true;
            for dr in 0..len {
                let row = start_row + dr;
                if state.is_unchecked(row, col) {
                    ok = false;
                    break;
                }
                if !state.is_checked(row, col) {
                    all_checked = false;
                }
            }
            if ok {
                possible += 1;
                if all_checked {
                    guaranteed += 1;
                }
            }
        }
    }

    // 主对角线方向（\）
    for start_row in 0..=size - len {
        for start_col in 0..=size - len {
            total += 1;
            let mut ok = true;
            let mut all_checked = true;
            for d in 0..len {
                let row = start_row + d;
                let col = start_col + d;
                if state.is_unchecked(row, col) {
                    ok = false;
                    break;
                }
                if !state.is_checked(row, col) {
                    all_checked = false;
                }
            }
            if ok {
                possible += 1;
                if all_checked {
                    guaranteed += 1;
                }
            }
        }
    }

    // 副对角线方向（/）
    for start_row in 0..=size - len {
        for start_col in (len - 1)..size {
            total += 1;
            let mut ok = true;
            let mut all_checked = true;
            for d in 0..len {
                let row = start_row + d;
                let col = start_col - d;
                if state.is_unchecked(row, col) {
                    ok = false;
                    break;
                }
                if !state.is_checked(row, col) {
                    all_checked = false;
                }
            }
            if ok {
                possible += 1;
                if all_checked {
                    guaranteed += 1;
                }
            }
        }
    }

    (total, possible, guaranteed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::masks::{cell_index, GRID_SIZE};
    use std::collections::BTreeSet;

    fn colors_from_rows(rows: [[Color; GRID_SIZE]; GRID_SIZE]) -> [Color; CELL_COUNT] {
        let mut out = [Color::White; CELL_COUNT];
        for x in 0..GRID_SIZE {
            for y in 0..GRID_SIZE {
                out[cell_index(x, y)] = rows[x][y];
            }
        }
        out
    }

    fn brute_force_solution_set(colors: [Color; CELL_COUNT]) -> BTreeSet<Mask> {
        let mut black_mask = 0u32;
        let mut vars = Vec::new();
        for i in 0..CELL_COUNT {
            if colors[i] == Color::Black {
                black_mask |= 1u32 << i;
            } else {
                vars.push(i);
            }
        }

        let color_grid: Vec<u8> = colors.iter().copied().map(Color::to_u8).collect();

        let mut set = BTreeSet::new();
        let combos = 1u64 << vars.len();
        for combo in 0..combos {
            let mut mask = black_mask;
            for (j, &i) in vars.iter().enumerate() {
                if (combo & (1u64 << j)) != 0 {
                    mask |= 1u32 << i;
                }
            }

            let res = crate::validate::validate_state(mask, &color_grid).expect("validate ok");
            if res.is_valid && res.is_bingo {
                set.insert(mask);
            }
        }
        set
    }

    #[test]
    fn solutions_match_bruteforce_for_fixed_cases() {
        let colors = colors_from_rows([
            [
                Color::Black,
                Color::Black,
                Color::Black,
                Color::Black,
                Color::Black,
            ],
            [
                Color::Black,
                Color::Red,
                Color::Blue,
                Color::Green,
                Color::Yellow,
            ],
            [
                Color::Black,
                Color::Purple,
                Color::Orange,
                Color::Cyan,
                Color::White,
            ],
            [
                Color::Black,
                Color::Black,
                Color::Black,
                Color::Black,
                Color::Black,
            ],
            [
                Color::Black,
                Color::Black,
                Color::Black,
                Color::Black,
                Color::Black,
            ],
        ]);

        let expected = brute_force_solution_set(colors);
        let got: BTreeSet<Mask> = Solver::new(colors)
            .solve_masks_limit(0)
            .into_iter()
            .collect();

        assert_eq!(got, expected);
    }
}
