use serde::Serialize;
use thiserror::Error;

use crate::colors::Color;
use crate::masks::{cell_bit, cell_index, BoardMasks, Mask};

#[derive(Debug, Error)]
pub enum ValidateError {
    #[error("color_grid 长度非法，得到：{0}")]
    BadGridLength(usize),
    #[error("color_grid 含非法颜色编码：index={index}, value={value}")]
    BadColor { index: usize, value: u8 },
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidateResult {
    pub is_bingo: bool,
    pub is_valid: bool,
    /// 每个格子的规则是否通过（row-major）。
    pub cell_ok: Vec<bool>,
    /// 每个格子的错误信息（若通过则为 None）。
    pub cell_messages: Vec<Option<String>>,
}

pub fn validate_state(
    checked_mask: Mask,
    color_grid: &[u8],
) -> Result<ValidateResult, ValidateError> {
    let cell_count = color_grid.len();
    let size = (cell_count as f64).sqrt() as usize;
    if size * size != cell_count {
        return Err(ValidateError::BadGridLength(cell_count));
    }

    let bm = BoardMasks::new(size);

    let mut colors = Vec::with_capacity(cell_count);
    for (i, &v) in color_grid.iter().enumerate() {
        colors.push(Color::from_u8(v).ok_or(ValidateError::BadColor { index: i, value: v })?);
    }

    let mut row_counts = vec![0u8; size];
    let mut col_counts = vec![0u8; size];
    for r in 0..size {
        row_counts[r] = (checked_mask & bm.row_masks[r]).count_ones() as u8;
    }
    for c in 0..size {
        col_counts[c] = (checked_mask & bm.col_masks[c]).count_ones() as u8;
    }

    let diag_count = bm.diag_down_masks.len();
    let mut diag_down_counts = vec![0u8; diag_count];
    let mut diag_up_counts = vec![0u8; diag_count];
    for d in 0..diag_count {
        diag_down_counts[d] = (checked_mask & bm.diag_down_masks[d]).count_ones() as u8;
        diag_up_counts[d] = (checked_mask & bm.diag_up_masks[d]).count_ones() as u8;
    }

    let mut cell_ok = vec![true; cell_count];
    let mut cell_messages = vec![None; cell_count];

    for x in 0..size {
        for y in 0..size {
            let i = cell_index(x, y, size);
            let (ok, msg) = match colors[i] {
                Color::Black => {
                    let ok = (checked_mask & cell_bit(i)) != 0;
                    (
                        ok,
                        if ok {
                            None
                        } else {
                            Some("黑格必须被勾选".to_string())
                        },
                    )
                }
                Color::White => (true, None),
                Color::Red => {
                    let count = (checked_mask & bm.neighbors_8[i]).count_ones();
                    let ok = count >= 1;
                    (
                        ok,
                        if ok {
                            None
                        } else {
                            Some(format!("周围需至少 1 个勾选，当前为 {}", count))
                        },
                    )
                }
                Color::Blue => {
                    let count = (checked_mask & bm.neighbors_8[i]).count_ones();
                    let ok = count <= 2;
                    (
                        ok,
                        if ok {
                            None
                        } else {
                            Some(format!("周围勾选不得超过 2 个，当前为 {}", count))
                        },
                    )
                }
                Color::Green => {
                    let r = row_counts[x];
                    let c = col_counts[y];
                    let ok = r == c;
                    (
                        ok,
                        if ok {
                            None
                        } else {
                            Some(format!("行勾选({}) 与 列勾选({}) 不相等", r, c))
                        },
                    )
                }
                Color::Yellow => {
                    let down_id = x + (size - 1) - y;
                    let up_id = x + y;
                    let d = diag_down_counts[down_id];
                    let u = diag_up_counts[up_id];
                    let ok = d == u;
                    (
                        ok,
                        if ok {
                            None
                        } else {
                            Some(format!("两条对角线勾选数不相等 ({} vs {})", d, u))
                        },
                    )
                }
                Color::Purple => {
                    let count = (checked_mask & bm.neighbors_8[i]).count_ones();
                    let ok = count % 2 == 1;
                    (
                        ok,
                        if ok {
                            None
                        } else {
                            Some(format!("周围勾选数需为奇数，当前为 {}", count))
                        },
                    )
                }
                Color::Orange => {
                    let count = (checked_mask & bm.neighbors_8[i]).count_ones();
                    let ok = count % 2 == 0;
                    (
                        ok,
                        if ok {
                            None
                        } else {
                            Some(format!("周围勾选数需为偶数，当前为 {}", count))
                        },
                    )
                }
                Color::Cyan => {
                    if (checked_mask & cell_bit(i)) == 0 {
                        (true, None)
                    } else {
                        let count = (checked_mask & bm.neighbors_4[i]).count_ones();
                        let ok = count >= 1;
                        (
                            ok,
                            if ok {
                                None
                            } else {
                                Some(format!(
                                    "被勾选时，上下左右需至少 1 个勾选，当前为 {}",
                                    count
                                ))
                            },
                        )
                    }
                }
            };
            cell_ok[i] = ok;
            cell_messages[i] = msg;
        }
    }

    let is_valid = cell_ok.iter().all(|&x| x);
    let is_bingo = bm.line_masks.iter().any(|&line| (checked_mask & line) == line);

    Ok(ValidateResult {
        is_bingo,
        is_valid,
        cell_ok,
        cell_messages,
    })
}
