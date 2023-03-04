use time::Date;
use super::Result;

pub fn check() {
    super::wrap_in_spinner(
        || is_working_day(super::today()),
        |r| if r { format!("Yay holiday!") } else { format!("No holiday today") }
    )
}

pub fn is_working_day(date: Date) -> Result<bool> {
    let session = super::get_session();
    let cal = crate::client::pto::holiday_calendar(&session)?;
    let found = match cal.into_iter().find(|hy| hy.year as i32 == date.year()) {
        Some(year) => year.holidays.into_iter().any(|h| h.start_date <= date && h.end_date >= date),
        None => false,
    };
    Ok(found)
}
