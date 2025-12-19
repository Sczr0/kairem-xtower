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

/// 生成 5x5 颜色布局（u8），满足：
/// - 白格比例 10%~20%（即 3~5 个）
/// - 至少存在一个解（满足规则且达成五连线）
pub fn generate_puzzle(seed: u64) -> Result<Vec<Vec<u8>>, GenerateError> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let max_attempts = 10_000usize;

    for _attempt in 0..max_attempts {
        let white_count = rng.gen_range(3usize..=5usize);
        let mut colors = [Color::White; CELL_COUNT];

        // 先填充非白色，避免白色比例失控。
        for x in 0..GRID_SIZE {
            for y in 0..GRID_SIZE {
                let i = cell_index(x, y);
                colors[i] = *NON_WHITE_COLORS.choose(&mut rng).expect("non-empty");
            }
        }

        // 随机挑选白格位置。
        let mut all_indices: Vec<usize> = (0..CELL_COUNT).collect();
        all_indices.shuffle(&mut rng);
        for &i in all_indices.iter().take(white_count) {
            colors[i] = Color::White;
        }

        // 至少存在一个解：用求解器快速查找一个解即可。
        let solver = Solver::new(colors);
        if solver.solve_masks_limit(1).is_empty() {
            continue;
        }

        let mut grid: Vec<Vec<u8>> = Vec::with_capacity(GRID_SIZE);
        for x in 0..GRID_SIZE {
            let mut row = Vec::with_capacity(GRID_SIZE);
            for y in 0..GRID_SIZE {
                row.push(colors[cell_index(x, y)].to_u8());
            }
            grid.push(row);
        }
        return Ok(grid);
    }

    Err(GenerateError::NoSatisfiablePuzzle {
        seed,
        attempts: max_attempts,
    })
}
