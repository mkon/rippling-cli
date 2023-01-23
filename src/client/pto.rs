use serde::Deserialize;
use serde_json::json;
use time::Date;

use super::session::Session;
use super::Result;

pub fn holiday_calendar(session: &Session) -> Result<Vec<HolidaysOfYear>> {
    let req = session.post(&format!("pto/api/get_holiday_calendar/"))?
        .json(&json!({"allow_time_admin": false, "only_payable": false}));
    super::request_to_result(req, |r| r.json::<Vec<HolidaysOfYear>>())
}

#[derive(Clone, Debug, Deserialize)]
pub struct HolidaysOfYear {
    pub year: u16,
    pub holidays: Vec<Holiday>
}

#[derive(Clone, Debug, Deserialize)]
pub struct Holiday {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(rename = "startDate")]
    pub start_date: Date,
    #[serde(rename = "endDate")]
    pub end_date: Date,
    #[serde(rename = "shouldCountTowardHoursWorkedForOvertime")]
    pub count_as_overtime: bool,
}

#[cfg(test)]
mod tests {
    use utilities::mocking;
    use time::macros::date;

    use super::*;

    fn session() -> Session {
        let mut session = Session::new("access-token".into());
        session.set_company_and_role("some-company-id".into(), "some-role-id".into());
        session
    }

    #[test]
    fn it_can_fetch_holiday_calendar() {
        let _m = mocking::with_fixture("POST", "/pto/api/get_holiday_calendar/", "holiday_calendar").create();
        let data = holiday_calendar(&session()).unwrap();
        assert_eq!(data.len(), 9);
        let y2023 = data.into_iter().find(|y| y.year == 2023).unwrap();
        assert_eq!(y2023.holidays.len(), 13);
        let days: Vec<Date> = y2023.holidays.into_iter().take(3).map(|h| h.start_date).collect();
        assert_eq!(days, vec![date![2023-01-01], date![2023-01-06], date![2023-04-07]]);
    }
}
