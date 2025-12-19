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

use wasm_bindgen::prelude::*;

pub use colors::{Color, COLOR_COUNT};
pub use difficulty::{DifficultyError, DifficultyReport};
pub use generate::GenerateError;
pub use validate::{ValidateError, ValidateResult};

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

/// Rust 原生接口：计算题目难度分（基于求解器搜索过程统计）。
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
