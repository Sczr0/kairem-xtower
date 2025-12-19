use thiserror::Error;

#[derive(Debug, Error)]
pub enum DateSeedError {
    #[error("日期格式错误：期望 YYYY-MM-DD，得到：{0}")]
    BadFormat(String),
    #[error("日期字段非法：{0}")]
    BadValue(String),
}

/// 将 YYYY-MM-DD 转为 u64 seed（以 UTC 1970-01-01 为 day0）。
///
/// 说明：
/// - 该 seed 仅用于“每日一题”的确定性输入，不依赖本地时区；
/// - 前端应使用 `Asia/Shanghai` 时区计算“今日日期字符串”，再传入此函数。
pub fn date_to_seed_ymd(date_ymd: &str) -> Result<u64, DateSeedError> {
    let parts: Vec<&str> = date_ymd.trim().split('-').collect();
    if parts.len() != 3 {
        return Err(DateSeedError::BadFormat(date_ymd.to_string()));
    }

    let year: i32 = parts[0]
        .parse()
        .map_err(|_| DateSeedError::BadValue(format!("year={}", parts[0])))?;
    let month: u32 = parts[1]
        .parse()
        .map_err(|_| DateSeedError::BadValue(format!("month={}", parts[1])))?;
    let day: u32 = parts[2]
        .parse()
        .map_err(|_| DateSeedError::BadValue(format!("day={}", parts[2])))?;

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(DateSeedError::BadValue(format!(
            "month/day out of range: {}-{:02}-{:02}",
            year, month, day
        )));
    }

    let days = days_from_civil(year, month, day);
    if days < 0 {
        return Err(DateSeedError::BadValue(format!(
            "date before 1970-01-01: {}",
            date_ymd
        )));
    }
    Ok(days as u64)
}

/// Howard Hinnant 的 civil-from-days 反推（返回 days since 1970-01-01）。
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = year as i64 - if month <= 2 { 1 } else { 0 };
    let m = month as i64;
    let d = day as i64;

    let era = if y >= 0 { y } else { y - 399 }.div_euclid(400);
    let yoe = y - era * 400;
    let mp = m + if m > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2).div_euclid(5) + d - 1;
    let doe = yoe * 365 + yoe.div_euclid(4) - yoe.div_euclid(100) + doy;

    // 719468 是 1970-01-01 的绝对 day offset
    era * 146097 + doe - 719468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_seed_is_days_since_epoch() {
        assert_eq!(date_to_seed_ymd("1970-01-01").unwrap(), 0);
        assert_eq!(date_to_seed_ymd("1970-01-02").unwrap(), 1);
        assert_eq!(date_to_seed_ymd("1970-01-31").unwrap(), 30);
    }
}
