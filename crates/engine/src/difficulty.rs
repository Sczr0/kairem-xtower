use serde::Serialize;
use thiserror::Error;

use crate::colors::Color;
use crate::masks::CELL_COUNT;
use crate::solver::{SolveStats, Solver};

#[derive(Debug, Error)]
pub enum DifficultyError {
    #[error("color_grid 长度必须为 {expected}，得到：{actual}")]
    BadGridLength { expected: usize, actual: usize },
    #[error("color_grid 含非法颜色编码：index={index}, value={value}")]
    BadColor { index: usize, value: u8 },
    #[error("该题目无解，无法计算难度分")]
    NoSolution,
}

#[derive(Debug, Clone, Serialize)]
pub struct DifficultyStats {
    pub node_visits: u32,
    pub decision_points: u32,
    pub branch_attempts: u32,
    pub dead_ends: u32,
    pub solutions: u32,

    pub propagate_rounds: u32,

    pub assignments_initial: u32,
    pub assignments_guess: u32,
    pub assignments_propagated: u32,

    pub max_depth: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DifficultyReport {
    pub difficulty_score: u32,
    pub stats: DifficultyStats,
}

fn difficulty_score_raw(stats: &SolveStats) -> u64 {
    let mut work = 0u64;
    work = work.saturating_add(stats.node_visits);
    work = work.saturating_add(stats.propagate_rounds);
    work = work.saturating_add(stats.assignments_propagated);
    work = work.saturating_add(stats.assignments_guess.saturating_mul(5));
    work = work.saturating_add(stats.dead_ends.saturating_mul(20));
    work = work.saturating_add((stats.max_depth as u64).saturating_mul(2));
    work
}

pub fn difficulty_report(color_grid: &[u8]) -> Result<DifficultyReport, DifficultyError> {
    if color_grid.len() != CELL_COUNT {
        return Err(DifficultyError::BadGridLength {
            expected: CELL_COUNT,
            actual: color_grid.len(),
        });
    }

    let mut colors = [Color::White; CELL_COUNT];
    for (i, &v) in color_grid.iter().enumerate() {
        colors[i] = Color::from_u8(v).ok_or(DifficultyError::BadColor { index: i, value: v })?;
    }

    let solver = Solver::new(colors);
    let mut stats = SolveStats::default();
    let solutions = solver.solve_masks_limit_with_stats(1, &mut stats);
    if solutions.is_empty() {
        return Err(DifficultyError::NoSolution);
    }

    let raw = difficulty_score_raw(&stats);
    let difficulty_score = raw.min(u32::MAX as u64) as u32;

    Ok(DifficultyReport {
        difficulty_score,
        stats: DifficultyStats {
            node_visits: stats.node_visits.min(u32::MAX as u64) as u32,
            decision_points: stats.decision_points.min(u32::MAX as u64) as u32,
            branch_attempts: stats.branch_attempts.min(u32::MAX as u64) as u32,
            dead_ends: stats.dead_ends.min(u32::MAX as u64) as u32,
            solutions: stats.solutions.min(u32::MAX as u64) as u32,
            propagate_rounds: stats.propagate_rounds.min(u32::MAX as u64) as u32,
            assignments_initial: stats.assignments_initial.min(u32::MAX as u64) as u32,
            assignments_guess: stats.assignments_guess.min(u32::MAX as u64) as u32,
            assignments_propagated: stats.assignments_propagated.min(u32::MAX as u64) as u32,
            max_depth: stats.max_depth,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_black_is_trivially_easy() {
        let grid = vec![Color::Black.to_u8(); CELL_COUNT];
        let report = difficulty_report(&grid).expect("should have solution");

        assert_eq!(report.stats.decision_points, 0);
        assert_eq!(report.stats.dead_ends, 0);
        assert!(report.difficulty_score > 0);
        assert!(report.difficulty_score < 100);
    }

    #[test]
    fn detects_no_solution() {
        // 让中心为 Blue，但其 8 邻域全是 Black（强制勾选），必然违反 Blue 的“邻域勾选数 <= 2”
        let mut grid = vec![Color::White.to_u8(); CELL_COUNT];
        grid[12] = Color::Blue.to_u8(); // (2,2) in 5x5 row-major
        for &i in &[6usize, 7, 8, 11, 13, 16, 17, 18] {
            grid[i] = Color::Black.to_u8();
        }

        let err = difficulty_report(&grid).expect_err("should be unsat");
        assert!(matches!(err, DifficultyError::NoSolution));
    }
}
