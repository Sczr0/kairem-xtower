/// 求解器状态（<=32x32）。
///
/// 设计要点：
/// - 每个格子是“勾选/不勾选/未知”三态；
/// - 用两套位图表达三态：
///   - `pos_*`：位为 1 表示“确定勾选”
///   - `neg_*`：位为 1 表示“确定不勾选”
///   - 两者都为 0 表示未知（禁止同一位同时为 1）
/// - 同时维护行视图与列视图，保证 O(1) 更新与 O(1) 行/列 min/max 计数推导。
#[derive(Clone, Debug)]
pub struct SolverState {
    size: usize,

    // 行视图：pos_rows[row] 的第 col 位为 1，表示 (row,col) 勾选
    pub(crate) pos_rows: Vec<u32>,
    pub(crate) neg_rows: Vec<u32>,

    // 列视图：pos_cols[col] 的第 row 位为 1，表示 (row,col) 勾选
    pub(crate) pos_cols: Vec<u32>,
    pub(crate) neg_cols: Vec<u32>,

    // 计数缓存：用于快速得到 [min,max] 范围
    row_checked: Vec<u8>,
    row_unchecked: Vec<u8>,
    col_checked: Vec<u8>,
    col_unchecked: Vec<u8>,

    diag_down_checked: Vec<u8>,
    diag_down_unchecked: Vec<u8>,
    diag_up_checked: Vec<u8>,
    diag_up_unchecked: Vec<u8>,
    diag_down_len: Vec<u8>,
    diag_up_len: Vec<u8>,
}

impl SolverState {
    pub fn new(size: usize) -> Self {
        assert!((1..=32).contains(&size), "size 必须在 1..=32，得到：{size}");

        let diag_count = size * 2 - 1;
        let mut diag_down_len = vec![0u8; diag_count];
        let mut diag_up_len = vec![0u8; diag_count];

        // 对角线长度只与 index 距离中心的偏移有关：
        // - down: row - col + (size-1)
        // - up:   row + col
        for id in 0..diag_count {
            let dist = (id as i32 - (size as i32 - 1)).abs() as usize;
            let len = (size - dist) as u8;
            diag_down_len[id] = len;
            diag_up_len[id] = len;
        }

        Self {
            size,
            pos_rows: vec![0; size],
            neg_rows: vec![0; size],
            pos_cols: vec![0; size],
            neg_cols: vec![0; size],
            row_checked: vec![0; size],
            row_unchecked: vec![0; size],
            col_checked: vec![0; size],
            col_unchecked: vec![0; size],
            diag_down_checked: vec![0; diag_count],
            diag_down_unchecked: vec![0; diag_count],
            diag_up_checked: vec![0; diag_count],
            diag_up_unchecked: vec![0; diag_count],
            diag_down_len,
            diag_up_len,
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline]
    fn valid_mask(&self) -> u32 {
        if self.size == 32 {
            u32::MAX
        } else {
            (1u32 << self.size) - 1
        }
    }

    #[inline]
    fn diag_down_id(&self, row: usize, col: usize) -> usize {
        row + (self.size - 1) - col
    }

    #[inline]
    fn diag_up_id(&self, row: usize, col: usize) -> usize {
        row + col
    }

    #[inline]
    fn id_to_rc(&self, id: usize) -> (usize, usize) {
        (id / self.size, id % self.size)
    }

    #[inline]
    pub fn is_checked(&self, row: usize, col: usize) -> bool {
        (self.pos_rows[row] & (1u32 << col)) != 0
    }

    #[inline]
    pub fn is_unchecked(&self, row: usize, col: usize) -> bool {
        (self.neg_rows[row] & (1u32 << col)) != 0
    }

    #[inline]
    pub fn is_unknown(&self, row: usize, col: usize) -> bool {
        !self.is_checked(row, col) && !self.is_unchecked(row, col)
    }

    #[inline]
    pub fn is_checked_id(&self, id: usize) -> bool {
        let (r, c) = self.id_to_rc(id);
        self.is_checked(r, c)
    }

    #[inline]
    pub fn is_unchecked_id(&self, id: usize) -> bool {
        let (r, c) = self.id_to_rc(id);
        self.is_unchecked(r, c)
    }

    #[inline]
    pub fn is_unknown_id(&self, id: usize) -> bool {
        let (r, c) = self.id_to_rc(id);
        self.is_unknown(r, c)
    }

    /// 设置 (row,col) 为“勾选”。
    ///
    /// - Ok(true)：成功且发生变化
    /// - Ok(false)：已是勾选（无变化）
    /// - Err(())：与“不勾选”矛盾
    pub fn set_checked(&mut self, row: usize, col: usize) -> Result<bool, ()> {
        let bit = 1u32 << col;
        if (self.neg_rows[row] & bit) != 0 {
            return Err(());
        }
        if (self.pos_rows[row] & bit) != 0 {
            return Ok(false);
        }

        self.pos_rows[row] |= bit;
        self.pos_cols[col] |= 1u32 << row;

        self.row_checked[row] += 1;
        self.col_checked[col] += 1;
        let down = self.diag_down_id(row, col);
        let up = self.diag_up_id(row, col);
        self.diag_down_checked[down] += 1;
        self.diag_up_checked[up] += 1;

        Ok(true)
    }

    /// 设置 (row,col) 为“不勾选”。
    ///
    /// - Ok(true)：成功且发生变化
    /// - Ok(false)：已是不勾选（无变化）
    /// - Err(())：与“勾选”矛盾
    pub fn set_unchecked(&mut self, row: usize, col: usize) -> Result<bool, ()> {
        let bit = 1u32 << col;
        if (self.pos_rows[row] & bit) != 0 {
            return Err(());
        }
        if (self.neg_rows[row] & bit) != 0 {
            return Ok(false);
        }

        self.neg_rows[row] |= bit;
        self.neg_cols[col] |= 1u32 << row;

        self.row_unchecked[row] += 1;
        self.col_unchecked[col] += 1;
        let down = self.diag_down_id(row, col);
        let up = self.diag_up_id(row, col);
        self.diag_down_unchecked[down] += 1;
        self.diag_up_unchecked[up] += 1;

        Ok(true)
    }

    #[inline]
    pub fn set_checked_id(&mut self, id: usize) -> Result<bool, ()> {
        let (r, c) = self.id_to_rc(id);
        self.set_checked(r, c)
    }

    #[inline]
    pub fn set_unchecked_id(&mut self, id: usize) -> Result<bool, ()> {
        let (r, c) = self.id_to_rc(id);
        self.set_unchecked(r, c)
    }

    /// 行的黑格（勾选）计数范围：[min,max]。
    #[inline]
    pub fn row_min_max(&self, row: usize) -> (u8, u8) {
        let min = self.row_checked[row];
        let max = (self.size as u8) - self.row_unchecked[row];
        (min, max)
    }

    /// 列的黑格（勾选）计数范围：[min,max]。
    #[inline]
    pub fn col_min_max(&self, col: usize) -> (u8, u8) {
        let min = self.col_checked[col];
        let max = (self.size as u8) - self.col_unchecked[col];
        (min, max)
    }

    /// down 对角线计数范围：[min,max]。
    #[inline]
    pub fn diag_down_min_max(&self, diag_id: usize) -> (u8, u8) {
        let min = self.diag_down_checked[diag_id];
        let max = self.diag_down_len[diag_id] - self.diag_down_unchecked[diag_id];
        (min, max)
    }

    /// up 对角线计数范围：[min,max]。
    #[inline]
    pub fn diag_up_min_max(&self, diag_id: usize) -> (u8, u8) {
        let min = self.diag_up_checked[diag_id];
        let max = self.diag_up_len[diag_id] - self.diag_up_unchecked[diag_id];
        (min, max)
    }

    /// 返回某一行的“未知列位图”（第 col 位为 1 表示未知）。
    #[inline]
    pub fn unknown_cols_mask_in_row(&self, row: usize) -> u32 {
        (!(self.pos_rows[row] | self.neg_rows[row])) & self.valid_mask()
    }

    /// 返回某一列的“未知行位图”（第 row 位为 1 表示未知）。
    #[inline]
    pub fn unknown_rows_mask_in_col(&self, col: usize) -> u32 {
        (!(self.pos_cols[col] | self.neg_cols[col])) & self.valid_mask()
    }

    /// 是否所有格子都已决定（勾选或不勾选）。
    pub fn is_fully_decided(&self) -> bool {
        let valid = self.valid_mask();
        for row in 0..self.size {
            let decided = (self.pos_rows[row] | self.neg_rows[row]) & valid;
            if decided != valid {
                return false;
            }
        }
        true
    }

    /// 用于“传播直到不再变化”的轻量 hash。
    pub fn hash64(&self) -> u64 {
        // FNV-1a（足够用于检测变更，不做安全用途）
        let mut h = 1469598103934665603u64;
        h ^= self.size as u64;
        h = h.wrapping_mul(1099511628211);

        for &v in &self.pos_rows {
            h ^= v as u64;
            h = h.wrapping_mul(1099511628211);
        }
        for &v in &self.neg_rows {
            h ^= v as u64;
            h = h.wrapping_mul(1099511628211);
        }
        h
    }

    /// 仅用于 5x5（或任意 size*size <= 32）将状态转为 row-major 的 u32 mask。
    pub fn to_row_major_u32_mask(&self) -> u32 {
        assert!(
            self.size * self.size <= 32,
            "to_row_major_u32_mask 仅支持 size*size<=32"
        );

        let mut out = 0u32;
        for row in 0..self.size {
            let bits = self.pos_rows[row] & self.valid_mask();
            for col in 0..self.size {
                if (bits & (1u32 << col)) != 0 {
                    let idx = row * self.size + col;
                    out |= 1u32 << idx;
                }
            }
        }
        out
    }
}

