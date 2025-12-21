//! Kairem-Xtower 引擎（WASM 导出入口）。
//!
//! - 颜色与规则语义：见仓库根目录 `rules.json`
//! - 位序：坐标 (0,0) 为 bit0（LSB），按行优先（row-major）排序

mod colors;
mod date_seed;
mod difficulty;
mod generate;
mod masks;
mod solver;
mod validate;

use serde::Serialize;
use wasm_bindgen::prelude::*;

pub use colors::{Color, COLOR_COUNT};
pub use difficulty::{DifficultyError, DifficultyReport};
pub use generate::GenerateError;
pub use solver::{HintAction, HintMove, HintResult, HintStatus};
pub use validate::{ValidateError, ValidateResult};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct SolutionCountResult {
    pub count: u32,
    pub truncated: bool,
}

fn parse_colors_5x5(color_grid: &[u8]) -> Result<[Color; crate::masks::CELL_COUNT], JsValue> {
    if color_grid.len() != crate::masks::CELL_COUNT {
        return Err(JsValue::from_str(&format!(
            "color_grid 长度必须为 25，得到：{}",
            color_grid.len()
        )));
    }

    let mut colors = [Color::White; crate::masks::CELL_COUNT];
    for (i, &v) in color_grid.iter().enumerate() {
        colors[i] = Color::from_u8(v).ok_or_else(|| {
            JsValue::from_str(&format!("color_grid 含非法颜色编码：index={i}, value={v}"))
        })?;
    }
    Ok(colors)
}

/// Rust 原生接口：生成 5x5 颜色布局（u8）。
pub fn generate_puzzle_grid(seed: u64) -> Result<Vec<Vec<u8>>, GenerateError> {
    generate::generate_puzzle(seed)
}

/// Rust 原生接口：校验当前状态与颜色布局。
pub fn validate_state_native(
    checked_mask: u32,
    color_grid: &[u8],
) -> Result<ValidateResult, ValidateError> {
    validate::validate_state(checked_mask, color_grid)
}

/// Rust 原生接口：计算题目难度分（偏向“人类逻辑难度”：传播 + 反证 + Bingo 约束强度）。
pub fn difficulty_report_native(color_grid: &[u8]) -> Result<DifficultyReport, DifficultyError> {
    difficulty::difficulty_report(color_grid)
}

#[wasm_bindgen]
pub fn date_to_seed_ymd(date_ymd: &str) -> Result<u64, JsValue> {
    date_seed::date_to_seed_ymd(date_ymd).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// 生成 5x5 颜色布局（确定性：同 seed 必然得到同一题）。
///
/// 返回值为可被 JS 直接使用的 JSON（`number[5][5]`，每个元素为颜色 u8）。
#[wasm_bindgen]
pub fn generate_puzzle(seed: u64) -> Result<JsValue, JsValue> {
    let grid = generate_puzzle_grid(seed).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_wasm_bindgen::to_value(&grid).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// 校验当前勾选状态与颜色布局。
///
/// - `checked_mask`：25 位 bitmask（u32），bit i 表示第 i 个格子是否勾选
/// - `color_grid`：长度必须为 25 的颜色数组（u8，row-major）
#[wasm_bindgen]
pub fn validate_state(checked_mask: u32, color_grid: Vec<u8>) -> Result<JsValue, JsValue> {
    let res = validate_state_native(checked_mask, &color_grid)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// 计算题目难度分（返回 JSON 对象）。
#[wasm_bindgen]
pub fn difficulty_report(color_grid: Vec<u8>) -> Result<JsValue, JsValue> {
    let report =
        difficulty_report_native(&color_grid).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_wasm_bindgen::to_value(&report).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// 给前端的“提示”接口：返回下一步建议/强制结论。
///
/// 约定：
/// - `checked_mask` 视为“已确认勾选”的集合，其余格子仍视为未知；
/// - 返回值为 JSON：`{status,message,move?}`。
#[wasm_bindgen]
pub fn hint_next(checked_mask: u32, color_grid: Vec<u8>) -> Result<JsValue, JsValue> {
    if color_grid.len() != crate::masks::CELL_COUNT {
        return Err(JsValue::from_str(&format!(
            "color_grid 长度必须为 25，得到：{}",
            color_grid.len()
        )));
    }

    let mut colors = [Color::White; crate::masks::CELL_COUNT];
    for (i, &v) in color_grid.iter().enumerate() {
        colors[i] = Color::from_u8(v).ok_or_else(|| {
            JsValue::from_str(&format!("color_grid 含非法颜色编码：index={i}, value={v}"))
        })?;
    }

    let solver = solver::Solver::new(colors);
    let res = solver.hint_next(checked_mask);
    serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// 统计解的数量（最多枚举到 `limit` 个解）。
///
/// 说明：
/// - 当 `limit=2` 时，足以区分：无解 / 唯一解 / 多解（>=2）；
/// - 返回的 `count` 不会超过 `limit`（当 `limit=0` 时表示不限制，并返回真实数量，但可能很慢）。
#[wasm_bindgen]
pub fn solution_count(color_grid: Vec<u8>, limit: u32) -> Result<JsValue, JsValue> {
    let colors = parse_colors_5x5(&color_grid)?;
    let solver = solver::Solver::new(colors);
    let solutions = solver.solve_masks_limit(limit as usize);
    let truncated = limit != 0 && (solutions.len() as u32) >= limit;
    let res = SolutionCountResult {
        count: solutions.len() as u32,
        truncated,
    };
    serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// 在“已有部分勾选”的前提下统计解数量（最多枚举到 `limit` 个解）。
#[wasm_bindgen]
pub fn solution_count_with_checked(
    checked_mask: u32,
    color_grid: Vec<u8>,
    limit: u32,
) -> Result<JsValue, JsValue> {
    let colors = parse_colors_5x5(&color_grid)?;
    let solver = solver::Solver::new(colors);
    let solutions = solver.solve_masks_limit_with_checked_mask(checked_mask, limit as usize);
    let truncated = limit != 0 && (solutions.len() as u32) >= limit;
    let res = SolutionCountResult {
        count: solutions.len() as u32,
        truncated,
    };
    serde_wasm_bindgen::to_value(&res).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(test)]
mod solution_count_tests {
    use super::*;

    #[test]
    fn solution_count_detects_unique_for_all_black() {
        let grid = vec![Color::Black.to_u8(); crate::masks::CELL_COUNT];
        let colors = parse_colors_5x5(&grid).expect("parse ok");
        let solver = crate::solver::Solver::new(colors);

        let solutions = solver.solve_masks_limit(2);
        assert_eq!(solutions.len(), 1);
    }

    #[test]
    fn solution_count_limit2_reports_multiple_when_possible() {
        // 第一行全黑：起始即满足 Bingo，其余格子为白色（几乎无约束），因此必然存在大量解
        let mut grid = vec![Color::White.to_u8(); crate::masks::CELL_COUNT];
        for i in 0..crate::masks::GRID_SIZE {
            grid[i] = Color::Black.to_u8();
        }

        let colors = parse_colors_5x5(&grid).expect("parse ok");
        let solver = crate::solver::Solver::new(colors);

        let solutions = solver.solve_masks_limit(2);
        assert_eq!(solutions.len(), 2);
    }
}
