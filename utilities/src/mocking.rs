pub use mockito::{Matcher, Mock};

pub struct FakeRippling {
    server: mockito::ServerGuard,
}

impl FakeRippling {
    pub fn new() -> Self {
        Self { server: mockito::Server::new() }
    }

    pub fn mock<P: Into<Matcher>>(&mut self, method: &str, path: P) -> Mock {
        self.server.mock(method, path)
    }

    pub fn url(&self) -> String {
        self.server.url()
    }

    fn default_mock<P: Into<Matcher>>(&mut self, method: &str, path: P) -> Mock {
        self.server
            .mock(method, path)
            .with_status(200)
            .with_header("content-type", "application/json")
            .match_header("authorization", "Bearer access-token")
    }

    pub fn with_fixture(&mut self, method: &str, path: &str, fixture: &str) -> Mock {
        let file = format!("{}/fixtures/{fixture}.json", env!("CARGO_MANIFEST_DIR"));
        self.default_mock(method, path).with_body_from_file(file)
    }

    pub fn mock_active_policy(&mut self) -> Mock {
        self.with_fixture(
            "GET",
            "/time_tracking/api/time_entry_policies/get_active_policy",
            "active_policy",
        )
        .create()
    }

    pub fn mock_break_policy(&mut self, id: &str) -> Mock {
        self.with_fixture(
            "GET",
            &format!("/time_tracking/api/time_entry_break_policies/{id}"),
            "break_policy",
        )
        .create()
    }
}
