use serde::Deserialize;
use serde_json::json;

use super::session::{Session, StatusCode};
use super::Result;

pub fn request(session: &Session, auth_option: &str) -> Result<MfaInfo> {
    let body = json!({ "authOption": auth_option });
    session
        .post("verification/api/identity_verification/request_authorization_code")
        .send_json(&body)?
        .accept_states(vec![StatusCode::OK, StatusCode::BAD_REQUEST])
        .parse_json::<MfaInfo>()
}

pub fn submit(session: &Session, auth_option: &str, code: &str) -> Result<MfaInfo> {
    let body = json!({"authOption": auth_option, "authorizationCode": code});
    session
        .post("verification/api/identity_verification/verify_authorization_code")
        .send_json(&body)?
        .accept_states(vec![StatusCode::OK, StatusCode::BAD_REQUEST])
        .parse_json::<MfaInfo>()
}

#[derive(Deserialize)]
pub struct MfaInfo {
    pub success: bool,
    pub message: String,
}
