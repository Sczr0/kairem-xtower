use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use thiserror::Error;

use crate::colors::{Color, NON_WHITE_COLORS};
use crate::masks::{cell_index, CELL_COUNT, GRID_SIZE};
use crate::solver::Solver;

#[derive(Debug, Error)]
pub enum GenerateError {
    #[error("在限制次数内未找到至少 1 个解的题目（seed={seed}, attempts={attempts}）")]
    NoSatisfiablePuzzle { seed: u64, attempts: usize },
}

/// 生成指定尺寸的颜色布局（u8），采用“反向构造”算法：
/// 1. 随机生成一个满足“五连线”的目标解 (Target Mask)。
/// 2. 根据目标解反推每个格子可用的颜色。
/// 3. 随机挑选 3~5 个格子设为白色。
pub fn generate_puzzle(seed: u64, size: usize) -> Result<Vec<Vec<u8>>, GenerateError> {
    use crate::masks::BoardMasks;

    let bm = BoardMasks::new(size);
    let cell_count = bm.cell_count;
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let max_attempts = 2000usize;

    for _attempt in 0..max_attempts {
        // --- 步骤 1: 构造目标解 ---
        let mut target_mask = *bm.line_masks.choose(&mut rng).unwrap();
        for i in 0..cell_count {
            if (target_mask & (1 << i)) == 0 && rng.gen_bool(0.4) {
                target_mask |= 1 << i;
            }
        }

        let mut row_counts = vec![0u8; size];
        let mut col_counts = vec![0u8; size];
        for r in 0..size {
            row_counts[r] = (target_mask & bm.row_masks[r]).count_ones() as u8;
        }
        for c in 0..size {
            col_counts[c] = (target_mask & bm.col_masks[c]).count_ones() as u8;
        }
        let diag_count = bm.diag_down_masks.len();
        let mut diag_down_counts = vec![0u8; diag_count];
        let mut diag_up_counts = vec![0u8; diag_count];
        for d in 0..diag_count {
            diag_down_counts[d] = (target_mask & bm.diag_down_masks[d]).count_ones() as u8;
            diag_up_counts[d] = (target_mask & bm.diag_up_masks[d]).count_ones() as u8;
        }

        // --- 步骤 2: 反向填充颜色 (多解融合) ---
        let mut colors = vec![Color::White; cell_count];
        for i in 0..cell_count {
            let x = i / size;
            let y = i % size;
            let is_checked = (target_mask & (1 << i)) != 0;

            let mut candidates = Vec::with_capacity(8);
            let n8_count = (target_mask & bm.neighbors_8[i]).count_ones();

            if n8_count >= 1 { candidates.push(Color::Red); }
            if n8_count <= 2 { candidates.push(Color::Blue); }
            if row_counts[x] == col_counts[y] { candidates.push(Color::Green); }
            let d_down = x + (size - 1) - y;
            let d_up = x + y;
            if diag_down_counts[d_down] == diag_up_counts[d_up] { candidates.push(Color::Yellow); }
            if n8_count % 2 == 1 { candidates.push(Color::Purple); }
            if n8_count % 2 == 0 { candidates.push(Color::Orange); }
            if !is_checked || (target_mask & bm.neighbors_4[i]).count_ones() >= 1 { candidates.push(Color::Cyan); }
            if is_checked { candidates.push(Color::Black); }

            if candidates.is_empty() {
                colors[i] = Color::White;
            } else {
                if rng.gen_bool(0.1) {
                    colors[i] = Color::White;
                } else {
                    colors[i] = *candidates.choose(&mut rng).unwrap();
                }
            }
        }

        // --- 步骤 3: 扰动填充 ---
        for _ in 0..5 {
            let idx = rng.gen_range(0..cell_count);
            let old_color = colors[idx];
            let new_color = *NON_WHITE_COLORS.choose(&mut rng).unwrap();
            colors[idx] = new_color;
            let solver = Solver::new(size, colors.clone());
            if solver.solve_masks_limit(1).is_empty() {
                colors[idx] = old_color;
            }
        }

        // --- 步骤 4: 随机挑选白格 ---
        let white_count = rng.gen_range(3usize..=5usize);
        let mut all_indices: Vec<usize> = (0..cell_count).collect();
        all_indices.shuffle(&mut rng);
        for &i in all_indices.iter().take(white_count) {
            colors[i] = Color::White;
        }

        // --- 步骤 5: 对称性变换 ---
        apply_symmetry(&mut colors, &mut rng, size);

        // --- 步骤 6: 校验解的存在性 ---
        let solver = Solver::new(size, colors.clone());
        if !solver.solve_masks_limit(1).is_empty() {
            let mut grid: Vec<Vec<u8>> = Vec::with_capacity(size);
            for x in 0..size {
                let mut row = Vec::with_capacity(size);
                for y in 0..size {
                    row.push(colors[x * size + y].to_u8());
                }
                grid.push(row);
            }
            return Ok(grid);
        }
    }

    Err(GenerateError::NoSatisfiablePuzzle {
        seed,
        attempts: max_attempts,
    })
}

/// 随机应用对称性变换（旋转、镜像）
fn apply_symmetry(colors: &mut Vec<Color>, rng: &mut ChaCha8Rng, size: usize) {
    let op = rng.gen_range(0..8);
    if op == 0 { return; } // Identity

    let mut new_colors = colors.clone();
    for x in 0..size {
        for y in 0..size {
            let (nx, ny) = match op {
                1 => (y, size - 1 - x), // Rot 90
                2 => (size - 1 - x, size - 1 - y), // Rot 180
                3 => (size - 1 - y, x), // Rot 270
                4 => (x, size - 1 - y), // Flip H
                5 => (size - 1 - x, y), // Flip V
                6 => (y, x), // Flip Main Diag
                7 => (size - 1 - y, size - 1 - x), // Flip Anti Diag
                _ => (x, y),
            };
            new_colors[nx * size + ny] = colors[x * size + y];
        }
    }
    *colors = new_colors;
}
