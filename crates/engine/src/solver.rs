use crate::colors::Color;
use crate::masks::{
    cell_bit, cell_index, Mask, CELL_COUNT, COL_MASKS, DIAG_COUNT, DIAG_DOWN_MASKS, DIAG_UP_MASKS,
    GRID_SIZE, LINE_MASKS, NEIGHBORS_4, NEIGHBORS_8, ROW_MASKS,
};

/// 求解器（回溯 + 剪枝）。
///
/// 说明：
/// - 黑格（Color::Black）强制勾选，因此不作为变量参与决策；
/// - 其余格子为变量，可选或不选；
/// - 解的判定：满足所有颜色规则且至少形成一条五连线。
pub struct Solver {
    colors: [Color; CELL_COUNT],
    pub(crate) black_mask: Mask,
    pub(crate) variable_mask: Mask,
    decision_order: Vec<usize>,
    blue_cells: Vec<usize>,
    red_cells: Vec<usize>,
    green_cells: Vec<(usize, usize)>,
    yellow_cells: Vec<(usize, usize)>,
    purple_cells: Vec<usize>,
    orange_cells: Vec<usize>,
    cyan_cells: Vec<usize>,
    blue_impacted_by_cell: [Mask; CELL_COUNT],
}

impl Solver {
    pub fn new(colors: [Color; CELL_COUNT]) -> Self {
        let mut black_mask = 0u32;
        for i in 0..CELL_COUNT {
            if colors[i] == Color::Black {
                black_mask |= cell_bit(i);
            }
        }
        let variable_mask = (!black_mask) & ((1u32 << CELL_COUNT) - 1);

        let mut blue_cells = Vec::new();
        let mut red_cells = Vec::new();
        let mut green_cells = Vec::new();
        let mut yellow_cells = Vec::new();
        let mut purple_cells = Vec::new();
        let mut orange_cells = Vec::new();
        let mut cyan_cells = Vec::new();

        for x in 0..GRID_SIZE {
            for y in 0..GRID_SIZE {
                let i = cell_index(x, y);
                match colors[i] {
                    Color::Blue => blue_cells.push(i),
                    Color::Red => red_cells.push(i),
                    Color::Green => green_cells.push((x, y)),
                    Color::Yellow => yellow_cells.push((x, y)),
                    Color::Purple => purple_cells.push(i),
                    Color::Orange => orange_cells.push(i),
                    Color::Cyan => cyan_cells.push(i),
                    _ => {}
                }
            }
        }

        let mut decision_order: Vec<usize> = (0..CELL_COUNT)
            .filter(|&i| (variable_mask & cell_bit(i)) != 0)
            .collect();
        decision_order.sort_by_key(|&i| {
            let neighbor_degree = NEIGHBORS_8[i].count_ones() as i32;
            let color_weight = match colors[i] {
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

        let mut blue_impacted_by_cell = [0u32; CELL_COUNT];
        for &blue in &blue_cells {
            let mut neighbors = NEIGHBORS_8[blue];
            while neighbors != 0 {
                let n = neighbors.trailing_zeros() as usize;
                neighbors &= neighbors - 1;
                blue_impacted_by_cell[n] |= cell_bit(blue);
            }
        }

        Self {
            colors,
            black_mask,
            variable_mask,
            decision_order,
            blue_cells,
            red_cells,
            green_cells,
            yellow_cells,
            purple_cells,
            orange_cells,
            cyan_cells,
            blue_impacted_by_cell,
        }
    }

    #[inline]
    fn has_five_in_a_row(checked: Mask) -> bool {
        LINE_MASKS.iter().any(|&line| (checked & line) == line)
    }

    fn check_all_rules(&self, checked: Mask) -> bool {
        let mut row_counts = [0u8; GRID_SIZE];
        let mut col_counts = [0u8; GRID_SIZE];
        for r in 0..GRID_SIZE {
            row_counts[r] = (checked & ROW_MASKS[r]).count_ones() as u8;
        }
        for c in 0..GRID_SIZE {
            col_counts[c] = (checked & COL_MASKS[c]).count_ones() as u8;
        }

        let mut diag_down_counts = [0u8; DIAG_COUNT];
        let mut diag_up_counts = [0u8; DIAG_COUNT];
        for d in 0..DIAG_COUNT {
            diag_down_counts[d] = (checked & DIAG_DOWN_MASKS[d]).count_ones() as u8;
            diag_up_counts[d] = (checked & DIAG_UP_MASKS[d]).count_ones() as u8;
        }

        for x in 0..GRID_SIZE {
            for y in 0..GRID_SIZE {
                let i = cell_index(x, y);
                match self.colors[i] {
                    Color::Black => {
                        if (checked & cell_bit(i)) == 0 {
                            return false;
                        }
                    }
                    Color::Red => {
                        if (checked & NEIGHBORS_8[i]) == 0 {
                            return false;
                        }
                    }
                    Color::Blue => {
                        if (checked & NEIGHBORS_8[i]).count_ones() > 2 {
                            return false;
                        }
                    }
                    Color::Green => {
                        if row_counts[x] != col_counts[y] {
                            return false;
                        }
                    }
                    Color::Yellow => {
                        let down_id = x + (GRID_SIZE - 1) - y;
                        let up_id = x + y;
                        if diag_down_counts[down_id] != diag_up_counts[up_id] {
                            return false;
                        }
                    }
                    Color::Purple => {
                        if (checked & NEIGHBORS_8[i]).count_ones() % 2 != 1 {
                            return false;
                        }
                    }
                    Color::Orange => {
                        if (checked & NEIGHBORS_8[i]).count_ones() % 2 != 0 {
                            return false;
                        }
                    }
                    Color::Cyan => {
                        if (checked & cell_bit(i)) != 0 && (checked & NEIGHBORS_4[i]) == 0 {
                            return false;
                        }
                    }
                    _ => {}
                }
            }
        }
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn partial_prune(
        &self,
        checked: Mask,
        decided_mask: Mask,
        decided_unchecked: Mask,
        row_checked: &[u8; GRID_SIZE],
        row_undecided: &[u8; GRID_SIZE],
        col_checked: &[u8; GRID_SIZE],
        col_undecided: &[u8; GRID_SIZE],
        diag_down_checked: &[u8; DIAG_COUNT],
        diag_down_undecided: &[u8; DIAG_COUNT],
        diag_up_checked: &[u8; DIAG_COUNT],
        diag_up_undecided: &[u8; DIAG_COUNT],
        blue_neighbor_checked: &[u8; CELL_COUNT],
    ) -> bool {
        if !LINE_MASKS
            .iter()
            .any(|&line| (line & decided_unchecked) == 0)
        {
            return false;
        }

        for &b in &self.blue_cells {
            if blue_neighbor_checked[b] > 2 {
                return false;
            }
        }

        for &(x, y) in &self.green_cells {
            let row_min = row_checked[x];
            let row_max = row_min + row_undecided[x];
            let col_min = col_checked[y];
            let col_max = col_min + col_undecided[y];
            if row_min > col_max || col_min > row_max {
                return false;
            }
        }

        for &(x, y) in &self.yellow_cells {
            let down_id = x + (GRID_SIZE - 1) - y;
            let up_id = x + y;
            let dmin = diag_down_checked[down_id];
            let dmax = dmin + diag_down_undecided[down_id];
            let umin = diag_up_checked[up_id];
            let umax = umin + diag_up_undecided[up_id];
            if dmin > umax || umin > dmax {
                return false;
            }
        }

        let undecided = self.variable_mask & !decided_mask;

        for &r in &self.red_cells {
            if (checked & NEIGHBORS_8[r]) != 0 {
                continue;
            }
            if (undecided & NEIGHBORS_8[r]) == 0 {
                return false;
            }
        }

        for &p in &self.purple_cells {
            let undecided_neighbors = undecided & NEIGHBORS_8[p];
            if undecided_neighbors == 0 {
                if (checked & NEIGHBORS_8[p]).count_ones() % 2 != 1 {
                    return false;
                }
            }
        }

        for &o in &self.orange_cells {
            let undecided_neighbors = undecided & NEIGHBORS_8[o];
            if undecided_neighbors == 0 {
                if (checked & NEIGHBORS_8[o]).count_ones() % 2 != 0 {
                    return false;
                }
            }
        }

        for &c in &self.cyan_cells {
            if (checked & cell_bit(c)) == 0 {
                continue;
            }
            if (checked & NEIGHBORS_4[c]) != 0 {
                continue;
            }
            if (undecided & NEIGHBORS_4[c]) == 0 {
                return false;
            }
        }

        true
    }

    /// 求解并返回最多 `limit` 个解（limit=0 视为不限制）。
    pub fn solve_masks_limit(&self, limit: usize) -> Vec<Mask> {
        let mut row_checked = [0u8; GRID_SIZE];
        let mut row_undecided = [0u8; GRID_SIZE];
        let mut col_checked = [0u8; GRID_SIZE];
        let mut col_undecided = [0u8; GRID_SIZE];
        let mut diag_down_checked = [0u8; DIAG_COUNT];
        let mut diag_down_undecided = [0u8; DIAG_COUNT];
        let mut diag_up_checked = [0u8; DIAG_COUNT];
        let mut diag_up_undecided = [0u8; DIAG_COUNT];

        for r in 0..GRID_SIZE {
            row_checked[r] = (self.black_mask & ROW_MASKS[r]).count_ones() as u8;
            row_undecided[r] = (self.variable_mask & ROW_MASKS[r]).count_ones() as u8;
        }
        for c in 0..GRID_SIZE {
            col_checked[c] = (self.black_mask & COL_MASKS[c]).count_ones() as u8;
            col_undecided[c] = (self.variable_mask & COL_MASKS[c]).count_ones() as u8;
        }
        for d in 0..DIAG_COUNT {
            diag_down_checked[d] = (self.black_mask & DIAG_DOWN_MASKS[d]).count_ones() as u8;
            diag_down_undecided[d] = (self.variable_mask & DIAG_DOWN_MASKS[d]).count_ones() as u8;
            diag_up_checked[d] = (self.black_mask & DIAG_UP_MASKS[d]).count_ones() as u8;
            diag_up_undecided[d] = (self.variable_mask & DIAG_UP_MASKS[d]).count_ones() as u8;
        }

        let mut blue_neighbor_checked = [0u8; CELL_COUNT];
        for &b in &self.blue_cells {
            blue_neighbor_checked[b] = (self.black_mask & NEIGHBORS_8[b]).count_ones() as u8;
            if blue_neighbor_checked[b] > 2 {
                return Vec::new();
            }
        }

        let mut solutions = Vec::new();
        self.backtrack(
            0,
            self.black_mask,
            0,
            row_checked,
            row_undecided,
            col_checked,
            col_undecided,
            diag_down_checked,
            diag_down_undecided,
            diag_up_checked,
            diag_up_undecided,
            blue_neighbor_checked,
            limit,
            &mut solutions,
        );
        solutions
    }

    #[allow(clippy::too_many_arguments)]
    fn backtrack(
        &self,
        idx: usize,
        checked: Mask,
        decided_mask: Mask,
        row_checked: [u8; GRID_SIZE],
        row_undecided: [u8; GRID_SIZE],
        col_checked: [u8; GRID_SIZE],
        col_undecided: [u8; GRID_SIZE],
        diag_down_checked: [u8; DIAG_COUNT],
        diag_down_undecided: [u8; DIAG_COUNT],
        diag_up_checked: [u8; DIAG_COUNT],
        diag_up_undecided: [u8; DIAG_COUNT],
        blue_neighbor_checked: [u8; CELL_COUNT],
        limit: usize,
        solutions: &mut Vec<Mask>,
    ) {
        if limit != 0 && solutions.len() >= limit {
            return;
        }

        if idx >= self.decision_order.len() {
            if Self::has_five_in_a_row(checked) && self.check_all_rules(checked) {
                solutions.push(checked);
            }
            return;
        }

        let cell = self.decision_order[idx];
        let bit = cell_bit(cell);

        // 分支1：不选
        {
            let decided2 = decided_mask | bit;
            let decided_unchecked2 = decided2 & !checked;

            let mut row_undecided2 = row_undecided;
            let mut col_undecided2 = col_undecided;
            let mut diag_down_undecided2 = diag_down_undecided;
            let mut diag_up_undecided2 = diag_up_undecided;

            let x = cell / GRID_SIZE;
            let y = cell % GRID_SIZE;
            row_undecided2[x] -= 1;
            col_undecided2[y] -= 1;
            diag_down_undecided2[x + (GRID_SIZE - 1) - y] -= 1;
            diag_up_undecided2[x + y] -= 1;

            if self.partial_prune(
                checked,
                decided2,
                decided_unchecked2,
                &row_checked,
                &row_undecided2,
                &col_checked,
                &col_undecided2,
                &diag_down_checked,
                &diag_down_undecided2,
                &diag_up_checked,
                &diag_up_undecided2,
                &blue_neighbor_checked,
            ) {
                self.backtrack(
                    idx + 1,
                    checked,
                    decided2,
                    row_checked,
                    row_undecided2,
                    col_checked,
                    col_undecided2,
                    diag_down_checked,
                    diag_down_undecided2,
                    diag_up_checked,
                    diag_up_undecided2,
                    blue_neighbor_checked,
                    limit,
                    solutions,
                );
            }
        }

        // 分支2：选择
        {
            let checked2 = checked | bit;
            let decided2 = decided_mask | bit;
            let decided_unchecked2 = decided2 & !checked2;

            let mut row_checked2 = row_checked;
            let mut row_undecided2 = row_undecided;
            let mut col_checked2 = col_checked;
            let mut col_undecided2 = col_undecided;
            let mut diag_down_checked2 = diag_down_checked;
            let mut diag_down_undecided2 = diag_down_undecided;
            let mut diag_up_checked2 = diag_up_checked;
            let mut diag_up_undecided2 = diag_up_undecided;

            let x = cell / GRID_SIZE;
            let y = cell % GRID_SIZE;
            row_checked2[x] += 1;
            row_undecided2[x] -= 1;
            col_checked2[y] += 1;
            col_undecided2[y] -= 1;
            let down_id = x + (GRID_SIZE - 1) - y;
            let up_id = x + y;
            diag_down_checked2[down_id] += 1;
            diag_down_undecided2[down_id] -= 1;
            diag_up_checked2[up_id] += 1;
            diag_up_undecided2[up_id] -= 1;

            let mut blue_neighbor_checked2 = blue_neighbor_checked;
            let impacted = self.blue_impacted_by_cell[cell];
            let mut m = impacted;
            while m != 0 {
                let b = m.trailing_zeros() as usize;
                m &= m - 1;
                blue_neighbor_checked2[b] += 1;
            }

            if self.partial_prune(
                checked2,
                decided2,
                decided_unchecked2,
                &row_checked2,
                &row_undecided2,
                &col_checked2,
                &col_undecided2,
                &diag_down_checked2,
                &diag_down_undecided2,
                &diag_up_checked2,
                &diag_up_undecided2,
                &blue_neighbor_checked2,
            ) {
                self.backtrack(
                    idx + 1,
                    checked2,
                    decided2,
                    row_checked2,
                    row_undecided2,
                    col_checked2,
                    col_undecided2,
                    diag_down_checked2,
                    diag_down_undecided2,
                    diag_up_checked2,
                    diag_up_undecided2,
                    blue_neighbor_checked2,
                    limit,
                    solutions,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let solver = Solver::new(colors);
        let mut vars = Vec::new();
        for i in 0..CELL_COUNT {
            if (solver.variable_mask & cell_bit(i)) != 0 {
                vars.push(i);
            }
        }

        let mut set = BTreeSet::new();
        let combos = 1u64 << vars.len();
        for combo in 0..combos {
            let mut mask = solver.black_mask;
            for (j, &i) in vars.iter().enumerate() {
                if (combo & (1u64 << j)) != 0 {
                    mask |= cell_bit(i);
                }
            }
            if Solver::has_five_in_a_row(mask) && solver.check_all_rules(mask) {
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
