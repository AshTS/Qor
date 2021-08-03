/// Standard structure for time storage
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RTCTime
{
    seconds: u32,
    minutes: u32,
    hours: u32,
    day: u32,
    month: u32,
    year: u32,
    unused: [u32; 3]
}

// This code is adapted from https://elixir.bootlin.com/linux/v5.13.7/source/drivers/rtc/lib.c#L49
impl RTCTime
{
    /// Get the number of leap days until the given year
    const fn get_leaps(year: i32) -> i32
    {
        year / 4 - year / 100 + year / 400
    }

    /// Check if a year is a leap year
    const fn check_leap_year(year: i32) -> bool
    {
        (year % 4 == 0) && ((year % 400 == 0) || (year % 100 != 0))
    }

    /// Number of days in a month
    const fn days_in_month(month: usize, year: i32) -> i32
    {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][month % 12] + 
            if month == 0 && Self::check_leap_year(year) { 1 } else { 0 }
    }

    /// Convert a Unix Timestamp (Seconds) into an RTCTime Object
    pub const fn unix_timestamp_to_rtc_time(unix: u64) -> Self
    {
        // TODO: This is part of a hack to make localization happen (THIS IS NOT
        // OKAY)
        let unix = unix as i64 + super::LOCALIZATION_OFFSET;

        let mut days: i32 = (unix / 24 / 3600) as i32;
        let mut year: i32 = 1970 + days as i32 / 365;
        let mut month: i32 = 0;

        days -= (year - 1970) * 365 + Self::get_leaps(year - 1) - Self::get_leaps(1970 - 1);

        while days < 0
        {
            year -= 1;
            days += 365 + if Self::check_leap_year(year) { 1 } else { 0 };
        }

        while month < 11
        {
            let next_days = days - Self::days_in_month(month as usize, year);

            if next_days < 0
            {
                break;
            }

            month += 1;
            days = next_days;
        }

        Self
        {
            seconds: (unix % 60)  as u32,
            minutes: ((unix / 60) % 60) as u32,
            hours: ((unix / 3600) % 24) as u32,
            day: 1 + days as u32,
            month: month as u32,
            year: year as u32 - 1900,
            unused: [0; 3]
        }
    }
}