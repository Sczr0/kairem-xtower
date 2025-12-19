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
}

#[derive(Clone, Copy, Debug)]
enum AssignReason {
    Initial,
    Guess,
    Propagate,
}

trait SolveObserver {
    fn on_node(&mut self, _depth: u32) {}
    fn on_decision_point(&mut self) {}
    fn on_branch_attempt(&mut self) {}
    fn on_dead_end(&mut self) {}
    fn on_solution(&mut self) {}
    fn on_propagate_round(&mut self) {}
    fn on_assignment(&mut self, _reason: AssignReason) {}
}

impl SolveObserver for () {}

impl SolveObserver for SolveStats {
    fn on_node(&mut self, depth: u32) {
        self.node_visits += 1;
        self.max_depth = self.max_depth.max(depth);
    }

    fn on_decision_point(&mut self) {
        self.decision_points += 1;
    }

    fn on_branch_attempt(&mut self) {
        self.branch_attempts += 1;
    }

    fn on_dead_end(&mut self) {
        self.dead_ends += 1;
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
            obs.on_dead_end();
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
        obs.on_decision_point();

        // 分支 1：先尝试“不勾选”（对很多规则更容易快速剪枝）
        {
            let mut fork = state.clone();
            obs.on_branch_attempt();
            if try_set_unchecked_id(&mut fork, cell, AssignReason::Guess, obs) {
                self.search(fork, limit, out, depth + 1, obs);
            } else {
                obs.on_dead_end();
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
                obs.on_dead_end();
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

            if !self.propagate_green(state, obs) {
                return false;
            }
            if !self.propagate_yellow(state, obs) {
                return false;
            }
            if !self.propagate_blue(state, obs) {
                return false;
            }
            if !self.propagate_red(state, obs) {
                return false;
            }
            if !self.propagate_purple(state, obs) {
                return false;
            }
            if !self.propagate_orange(state, obs) {
                return false;
            }
            if !self.propagate_cyan(state, obs) {
                return false;
            }
            if !self.propagate_five_in_a_row_possible(state) {
                return false;
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
