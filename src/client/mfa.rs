use serde::Deserialize;
use serde_json::json;

use super::session::Session;
use super::Result;

pub fn request(session: &Session, auth_option: &str) -> Result<MfaInfo> {
    let body = json!({ "authOption": auth_option });
    let req = session
        .post("verification/api/identity_verification/request_authorization_code")?
        .json(&body);
    let res = req.send()?;
    match res.status() {
        reqwest::StatusCode::OK => res.json::<MfaInfo>().map_err(super::Error::from),
        reqwest::StatusCode::BAD_REQUEST => res.json::<MfaInfo>().map_err(super::Error::from),
        _ => Err(res.into()),
    }
}

pub fn submit(session: &Session, auth_option: &str, code: &str) -> Result<MfaInfo> {
    let body = json!({"authOption": auth_option, "authorizationCode": code});
    let req = session
        .post("verification/api/identity_verification/verify_authorization_code")?
        .json(&body);
    let res = req.send()?;
    match res.status() {
        reqwest::StatusCode::OK => res.json::<MfaInfo>().map_err(super::Error::from),
        reqwest::StatusCode::BAD_REQUEST => res.json::<MfaInfo>().map_err(super::Error::from),
        _ => Err(res.into()),
    }
}

#[derive(Deserialize)]
pub struct MfaInfo {
    pub success: bool,
    pub message: String,
}
