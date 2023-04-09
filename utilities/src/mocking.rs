pub use mockito::{mock, server_url, Matcher, Mock};

pub fn rippling(method: &str, path: &str) -> Mock {
    mock(method, path)
        .with_status(200)
        .with_header("content-type", "application/json")
        .match_header("authorization", "Bearer access-token")
}

pub fn with_fixture(method: &str, path: &str, fixture: &str) -> Mock {
    let file = format!("{}/fixtures/{fixture}.json", env!("CARGO_MANIFEST_DIR"));
    rippling(method, path).with_body_from_file(file)
}

pub fn mock_active_policy() -> Mock {
    with_fixture(
        "GET",
        "/time_tracking/api/time_entry_policies/get_active_policy",
        "active_policy",
    )
    .create()
}

pub fn mock_break_policy(id: &str) -> Mock {
    with_fixture(
        "GET",
        &format!("/time_tracking/api/time_entry_break_policies/{id}"),
        "break_policy",
    )
    .create()
}
