use crate::IntegrationTestsOpts;
use methods::{JsonRpcRequest, JsonRpcVersion};

pub struct IntegrationTests {
    opts: IntegrationTestsOpts,
    client: reqwest::Client,
}

impl IntegrationTests {
    pub fn new(opts: IntegrationTestsOpts) -> Self {
        let client = reqwest::Client::new();
        Self { opts, client }
    }

    pub async fn run_tests(&self) {
        info!("Adding user 'test_user'...");
        self.test_add_user().await.unwrap();
    }

    async fn test_add_user(&self) -> Result<(), ()> {
        let params = methods::AddUserParams::new(methods::User::new(
            "test_user".into(),
            "test_password".into(),
        ));
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            methods::Method::AddUser.to_string(),
            serde_json::to_value(params).unwrap(),
            Some("mngr_test_add_user".into()),
        );

        let response: serde_json::Value = self
            .client
            .post(&self.opts.url)
            .body(serde_json::to_string(&request).unwrap())
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        info!("response: '{:?}'", response);

        Ok(())
    }
}
