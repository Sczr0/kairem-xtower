use once_cell::sync::Lazy;

pub type Mask = u32;

pub const GRID_SIZE: usize = 5;
pub const CELL_COUNT: usize = GRID_SIZE * GRID_SIZE;
pub const DIAG_COUNT: usize = GRID_SIZE * 2 - 1;
pub const LINE_COUNT: usize = GRID_SIZE * 2 + 2;

#[inline]
pub fn cell_index(x: usize, y: usize) -> usize {
    x * GRID_SIZE + y
}

#[inline]
pub fn cell_bit(i: usize) -> Mask {
    1u32 << i
}

pub static NEIGHBORS_8: Lazy<[Mask; CELL_COUNT]> = Lazy::new(|| {
    let mut masks = [0u32; CELL_COUNT];
    for x in 0..GRID_SIZE {
        for y in 0..GRID_SIZE {
            let mut m = 0u32;
            for nx in x.saturating_sub(1)..=(x + 1).min(GRID_SIZE - 1) {
                for ny in y.saturating_sub(1)..=(y + 1).min(GRID_SIZE - 1) {
                    if nx == x && ny == y {
                        continue;
                    }
                    m |= cell_bit(cell_index(nx, ny));
                }
            }
            masks[cell_index(x, y)] = m;
        }
    }
    masks
});

pub static NEIGHBORS_4: Lazy<[Mask; CELL_COUNT]> = Lazy::new(|| {
    let mut masks = [0u32; CELL_COUNT];
    for x in 0..GRID_SIZE {
        for y in 0..GRID_SIZE {
            let mut m = 0u32;
            if x > 0 {
                m |= cell_bit(cell_index(x - 1, y));
            }
            if x + 1 < GRID_SIZE {
                m |= cell_bit(cell_index(x + 1, y));
            }
            if y > 0 {
                m |= cell_bit(cell_index(x, y - 1));
            }
            if y + 1 < GRID_SIZE {
                m |= cell_bit(cell_index(x, y + 1));
            }
            masks[cell_index(x, y)] = m;
        }
    }
    masks
});

pub static ROW_MASKS: Lazy<[Mask; GRID_SIZE]> = Lazy::new(|| {
    let mut masks = [0u32; GRID_SIZE];
    for r in 0..GRID_SIZE {
        let mut m = 0u32;
        for c in 0..GRID_SIZE {
            m |= cell_bit(cell_index(r, c));
        }
        masks[r] = m;
    }
    masks
});

pub static COL_MASKS: Lazy<[Mask; GRID_SIZE]> = Lazy::new(|| {
    let mut masks = [0u32; GRID_SIZE];
    for c in 0..GRID_SIZE {
        let mut m = 0u32;
        for r in 0..GRID_SIZE {
            m |= cell_bit(cell_index(r, c));
        }
        masks[c] = m;
    }
    masks
});

pub static DIAG_DOWN_MASKS: Lazy<[Mask; DIAG_COUNT]> = Lazy::new(|| {
    let mut masks = [0u32; DIAG_COUNT];
    for x in 0..GRID_SIZE {
        for y in 0..GRID_SIZE {
            let id = x + (GRID_SIZE - 1) - y;
            masks[id] |= cell_bit(cell_index(x, y));
        }
    }
    masks
});

pub static DIAG_UP_MASKS: Lazy<[Mask; DIAG_COUNT]> = Lazy::new(|| {
    let mut masks = [0u32; DIAG_COUNT];
    for x in 0..GRID_SIZE {
        for y in 0..GRID_SIZE {
            let id = x + y;
            masks[id] |= cell_bit(cell_index(x, y));
        }
    }
    masks
});

pub static LINE_MASKS: Lazy<[Mask; LINE_COUNT]> = Lazy::new(|| {
    let mut masks = [0u32; LINE_COUNT];
    for r in 0..GRID_SIZE {
        masks[r] = ROW_MASKS[r];
    }
    for c in 0..GRID_SIZE {
        masks[GRID_SIZE + c] = COL_MASKS[c];
    }

    let mut diag_main = 0u32;
    let mut diag_anti = 0u32;
    for k in 0..GRID_SIZE {
        diag_main |= cell_bit(cell_index(k, k));
        diag_anti |= cell_bit(cell_index(k, GRID_SIZE - 1 - k));
    }
    masks[GRID_SIZE * 2] = diag_main;
    masks[GRID_SIZE * 2 + 1] = diag_anti;
    masks
});
