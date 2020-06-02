use crate::IntegrationTestsOpts;

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
        let user_request = r#"
            {
                "jsonrpc": "2.0",
                "method": "add_user",
                "id": "mngr_test_add_user",
                "params": {
                    "user": {
                        "username": "test_user",
                        "password": "test_password"
                    }
                }
            }
        "#;

        let response = self
            .client
            .post(&format!("{}:{}", self.opts.url, self.opts.port))
            .body(user_request)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        info!("response: '{}'", response);

        Ok(())
    }
}
