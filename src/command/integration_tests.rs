use crate::IntegrationTestsOpts;
use webserver_client::WebserverClient;
use webserver_contracts::{
    prediction::AddPredictionParams,
    user::{AddUserParams, DeleteUserParams, User},
    JsonRpcRequest, JsonRpcResponse, JsonRpcVersion, Method, ResponseKind,
};

pub struct IntegrationTests {
    opts: IntegrationTestsOpts,
    client: WebserverClient,
    test_user: User,
}

impl IntegrationTests {
    pub fn new(opts: IntegrationTestsOpts) -> Self {
        let client = WebserverClient::new()
            .with_url(opts.url.clone())
            .build()
            .unwrap();
        let test_user = User::new("test_user".to_string(), "test_password".to_string());
        Self {
            opts,
            client,
            test_user,
        }
    }

    pub async fn run_tests(&self) {
        info!("Adding user 'test_user'...");
        self.add_user().await.unwrap();
        info!("Adding a prediction by 'test_user'...");
        self.add_prediction().await.unwrap();
        info!("Deleting user 'test_user'...");
        self.delete_user().await;
    }

    async fn add_user(&self) -> Result<(), String> {
        let params = AddUserParams::new(User::new("test_user".into(), "test_password".into()));
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::AddUser.to_string(),
            serde_json::to_value(params).unwrap(),
            Some("mngr_test_add_user".into()),
        );

        let response: JsonRpcResponse = self
            .client
            .send_request(request)
            .await
            .map_err(|e| format!("{:?}", e))?;

        match response.kind() {
            ResponseKind::Success => Ok(()),
            ResponseKind::Error(e) => Err(format!("{:?}", e)),
        }
    }

    async fn add_prediction(&self) -> Result<(), String> {
        let params = AddPredictionParams::new("Test prediction".into(), self.test_user.clone());
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::AddPrediction.to_string(),
            params,
            Some("mngr_test_add_prediction".into()),
        );

        let response: JsonRpcResponse = self.client.send_request(request).await.unwrap();

        match response.kind() {
            ResponseKind::Success => Ok(()),
            ResponseKind::Error(e) => Err(format!("{:?}", e)),
        }
    }

    async fn delete_user(&self) {
        let params = DeleteUserParams::new(
            User::new("test_user".into(), "test_password".into()),
            "test_user".into(),
        );
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::DeleteUser.to_string(),
            params,
            Some("mngr_test_delete_user".into()),
        );

        let response: JsonRpcResponse = self.client.send_request(request).await.unwrap();

        match response.kind() {
            ResponseKind::Success => info!("success response"),
            ResponseKind::Error(e) => error!("{:?}", e),
        }
    }
}
