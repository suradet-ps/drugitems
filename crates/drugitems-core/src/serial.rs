//! Excel serial date conversion.
//!
//! `.xls` exports store dates as serial numbers: whole days since
//! 1899-12-30 plus a fraction of a day for the time (e.g. `44961.6197...`
//! is `2023-01-31 14:52:...`). These helpers recover real
//! [`NaiveDateTime`]s so date columns can be compared with the database
//! values instead of raw floats.

use chrono::{Days, NaiveDate, NaiveDateTime, NaiveTime};

/// Number of seconds in a day.
const SECS_PER_DAY: f64 = 86_400.0;

/// The epoch used by Excel: 1899-12-30 (the "1900 date system").
fn excel_epoch() -> NaiveDate {
    NaiveDate::from_ymd_opt(1899, 12, 30).expect("invariant: fixed date is valid")
}

/// Convert an Excel serial number to a date-time.
///
/// Returns `None` for non-finite or absurdly small serials (which cannot be
/// dates in practice; a serial below 60 is a 1900 artifact and out of scope
/// for a drug master).
pub fn serial_to_datetime(serial: f64) -> Option<NaiveDateTime> {
    if !serial.is_finite() || serial < 60.0 {
        return None;
    }
    let days = serial.floor() as i64;
    let mut secs = ((serial - days as f64) * SECS_PER_DAY).round() as i64;
    let mut date = excel_epoch().checked_add_days(Days::new(days as u64))?;
    if secs >= SECS_PER_DAY as i64 {
        date = date.checked_add_days(Days::new(1))?;
        secs -= SECS_PER_DAY as i64;
    }
    let time = NaiveTime::from_num_seconds_from_midnight_opt(secs as u32, 0)?;
    Some(date.and_time(time))
}

/// Convert an Excel serial number to a calendar date (time discarded).
pub fn serial_to_date(serial: f64) -> Option<NaiveDate> {
    serial_to_datetime(serial).map(|dt| dt.date())
}

/// Convert an Excel serial number to a clock time (day fraction).
pub fn serial_to_time(serial: f64) -> Option<NaiveTime> {
    if !serial.is_finite() {
        return None;
    }
    // Excel may store a time as a fraction of a day (0..1) or as an
    // absolute serial; take the fractional part either way.
    let fraction = serial - serial.floor();
    let secs = (fraction * SECS_PER_DAY).round() as i64;
    NaiveTime::from_num_seconds_from_midnight_opt(secs.min(86_399) as u32, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn serial_to_datetime_recovers_known_value() {
        // 44961 days after 1899-12-30 is 2023-02-04; 0.61979... of a day is
        // ~14:52:30. We assert the date and a plausible time-of-day.
        let dt = serial_to_datetime(44961.619791666664).expect("serial must convert");
        assert_eq!(dt.date(), NaiveDate::from_ymd_opt(2023, 2, 4).unwrap());
        assert_eq!(dt.time().num_seconds_from_midnight(), 53_550);
    }

    #[test]
    fn serial_to_date_discards_time() {
        let d = serial_to_date(44961.5).expect("serial must convert");
        assert_eq!(d, NaiveDate::from_ymd_opt(2023, 2, 4).unwrap());
    }

    #[test]
    fn serial_to_time_uses_day_fraction() {
        let t = serial_to_time(0.5).expect("half day");
        assert_eq!(t, NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    }

    #[test]
    fn non_finite_or_tiny_serial_rejected() {
        assert!(serial_to_datetime(f64::NAN).is_none());
        assert!(serial_to_datetime(0.0).is_none());
        assert!(serial_to_date(1.0).is_none());
    }
}
