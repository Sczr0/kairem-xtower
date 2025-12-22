use once_cell::sync::Lazy;

pub type Mask = u64;

pub const GRID_SIZE: usize = 5;
pub const CELL_COUNT: usize = GRID_SIZE * GRID_SIZE;

#[inline]
pub fn cell_index(x: usize, y: usize, size: usize) -> usize {
    x * size + y
}

#[inline]
pub fn cell_bit(i: usize) -> Mask {
    1u64 << i
}

pub struct BoardMasks {
    pub size: usize,
    pub cell_count: usize,
    pub neighbors_8: Vec<Mask>,
    pub neighbors_4: Vec<Mask>,
    pub row_masks: Vec<Mask>,
    pub col_masks: Vec<Mask>,
    pub diag_down_masks: Vec<Mask>,
    pub diag_up_masks: Vec<Mask>,
    pub line_masks: Vec<Mask>,
}

impl BoardMasks {
    pub fn new(size: usize) -> Self {
        let cell_count = size * size;
        let diag_count = if size > 0 { size * 2 - 1 } else { 0 };
        let line_count = size * 2 + 2;

        let mut neighbors_8 = vec![0u64; cell_count];
        let mut neighbors_4 = vec![0u64; cell_count];
        for x in 0..size {
            for y in 0..size {
                let idx = cell_index(x, y, size);
                let mut m8 = 0u64;
                let mut m4 = 0u64;
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < size as i32 && ny >= 0 && ny < size as i32 {
                            let n_idx = cell_index(nx as usize, ny as usize, size);
                            m8 |= cell_bit(n_idx);
                            if dx.abs() + dy.abs() == 1 {
                                m4 |= cell_bit(n_idx);
                            }
                        }
                    }
                }
                neighbors_8[idx] = m8;
                neighbors_4[idx] = m4;
            }
        }

        let mut row_masks = vec![0u64; size];
        for r in 0..size {
            let mut m = 0u64;
            for c in 0..size {
                m |= cell_bit(cell_index(r, c, size));
            }
            row_masks[r] = m;
        }

        let mut col_masks = vec![0u64; size];
        for c in 0..size {
            let mut m = 0u64;
            for r in 0..size {
                m |= cell_bit(cell_index(r, c, size));
            }
            col_masks[c] = m;
        }

        let mut diag_down_masks = vec![0u64; diag_count];
        let mut diag_up_masks = vec![0u64; diag_count];
        for x in 0..size {
            for y in 0..size {
                let d_down = x + (size - 1) - y;
                let d_up = x + y;
                diag_down_masks[d_down] |= cell_bit(cell_index(x, y, size));
                diag_up_masks[d_up] |= cell_bit(cell_index(x, y, size));
            }
        }

        let mut line_masks = vec![0u64; line_count];
        for r in 0..size {
            line_masks[r] = row_masks[r];
        }
        for c in 0..size {
            line_masks[size + c] = col_masks[c];
        }
        let mut diag_main = 0u64;
        let mut diag_anti = 0u64;
        for k in 0..size {
            diag_main |= cell_bit(cell_index(k, k, size));
            diag_anti |= cell_bit(cell_index(k, size - 1 - k, size));
        }
        line_masks[size * 2] = diag_main;
        line_masks[size * 2 + 1] = diag_anti;

        Self {
            size,
            cell_count,
            neighbors_8,
            neighbors_4,
            row_masks,
            col_masks,
            diag_down_masks,
            diag_up_masks,
            line_masks,
        }
    }
}
