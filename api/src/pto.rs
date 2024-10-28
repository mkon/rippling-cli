use serde::Deserialize;
use serde_json::json;
use time::Date;

use super::Result;

impl crate::Client {
    pub fn holiday_calendar(&self) -> Result<Vec<HolidaysOfYear>> {
        let holidays: Vec<HolidaysOfYear> = self
            .post("pto/api/get_holiday_calendar/")
            .send_json(json!({"allow_time_admin": false, "only_payable": false}))?
            .into_json()?;
        Ok(holidays)
    }

    pub fn leave_requests(&self) -> Result<Vec<LeaveRequest>> {
        let role = self.role().unwrap();
        let query: Vec<(&str, &str)> = vec![("role", role), ("status", "APPROVED")];
        let requests: Vec<LeaveRequest> = self
            .get("pto/api/leave_requests/")
            .query_pairs(query)
            .call()?
            .into_json()?;
        Ok(requests)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct HolidaysOfYear {
    pub year: u16,
    pub holidays: Vec<Holiday>,
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

#[derive(Clone, Debug, Deserialize)]
pub struct LeaveRequest {
    #[serde(rename = "isDeleted")]
    pub is_deleted: Option<bool>,
    #[serde(rename = "startDate")]
    pub start_date: Date,
    #[serde(rename = "endDate")]
    pub end_date: Date,
    pub status: String,
    #[serde(rename = "leaveTypeName")]
    pub leave_type_name: String,
}

#[cfg(test)]
mod tests {
    use time::macros::date;
    use utilities::mocking;

    use crate::Client;

    use super::*;

    fn setup() -> (mocking::FakeRippling, Client) {
        let server = mocking::FakeRippling::new();
        let client = Client::new("access-token".to_owned())
            .with_root(url::Url::parse(&server.url()).unwrap())
            .with_company_and_role("some-company-id".to_owned(), "some-role-id".to_owned());
        (server, client)
    }

    #[test]
    fn it_can_fetch_leave_requests() {
        let (mut server, client) = setup();
        let _m = server
            .with_fixture(
                "GET",
                "/pto/api/leave_requests/?role=some-role-id&status=APPROVED",
                "leave_requests",
            )
            .create();
        let data = client.leave_requests().unwrap();
        assert_eq!(data.len(), 2);
        let days: Vec<Date> = data.into_iter().map(|h| h.start_date).collect();
        assert_eq!(days, vec![date![2022 - 06 - 09], date![2022 - 05 - 23]]);
    }

    #[test]
    fn it_can_fetch_holiday_calendar() {
        let (mut server, client) = setup();
        let _m = server
            .with_fixture("POST", "/pto/api/get_holiday_calendar/", "holiday_calendar")
            .create();
        let data = client.holiday_calendar().unwrap();
        assert_eq!(data.len(), 9);
        let y2023 = data.into_iter().find(|y| y.year == 2023).unwrap();
        assert_eq!(y2023.holidays.len(), 13);
        let days: Vec<Date> = y2023.holidays.into_iter().take(3).map(|h| h.start_date).collect();
        assert_eq!(
            days,
            vec![date![2023 - 01 - 01], date![2023 - 01 - 06], date![2023 - 04 - 07]]
        );
    }
}
