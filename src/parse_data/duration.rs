use crate::{
    err_new,
    error::{Error, Kind, Result},
};
use std::time::{SystemTime, UNIX_EPOCH};
fn days_since_epoch(mut year: i16, mut month: u8, day: u8) -> Result<i64> {
    // 参数有效性验证
    if !(1..=12).contains(&month) || day == 0 || day > 31 {
        return Err("Invalid date".into());
    }

    // 转换年为i32处理BC日期
    let is_bc = year < 1;
    if is_bc {
        year = 1 - year;
    }

    // 调整年月为计算格式
    if month < 3 {
        month += 12;
        year -= 1;
    }

    // 使用Zeller公式计算绝对天数
    let a = year / 100;
    let b = a / 4;
    let c = 2 - a + b;
    let e = (365.25 * (year + 4716) as f64) as i64;
    let f = (30.6001 * (month as f64 + 1.0)) as i64;
    let jd = c as i64 + day as i64 + e + f - 1524;

    // 减去UNIX纪元的天数（2440588）
    let unix_epoch_jd = 2440588;
    let days = jd - unix_epoch_jd;

    // 处理BC日期符号
    Ok(if is_bc { -days } else { days })
}

fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn now_since_epoch() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before Unix epoch")
        .as_secs();

    now / 86400 // 86400秒 = 1天
}

// 计算两个日期之间的天数
pub fn days_between_dates((year, month, day): (i16, u8, u8)) -> Result<i64> {
    let now_since_epoch = now_since_epoch();
    let given_day_since_epoch = days_since_epoch(year, month, day)?;

    Ok(i64::try_from(now_since_epoch)? - given_day_since_epoch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let year = 0;
        let month = 1;
        let day = 1;
        let days_since_epoch = days_since_epoch(year, month, day).unwrap();
        dbg!(days_since_epoch);
    }
}
