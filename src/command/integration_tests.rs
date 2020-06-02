use crate::IntegrationTestsOpts;
use webserver_client::WebserverClient;
use webserver_contracts::{
    user::{DeleteUserParams, SetRoleParams, User},
    JsonRpcRequest, JsonRpcResponse, JsonRpcVersion, Method, ResponseKind,
};

pub struct IntegrationTests {
    opts: IntegrationTestsOpts,
    client: WebserverClient,
}

impl IntegrationTests {
    pub fn new(opts: IntegrationTestsOpts) -> Self {
        let client = WebserverClient::new()
            .with_url(opts.url.clone())
            .build()
            .unwrap();
        Self { opts, client }
    }

    pub async fn run_tests(&self) {
        info!("Adding user 'test_user'...");
        self.test_add_user().await;
        info!("Deleting user 'test_user'...");
        self.test_delete_user().await;
    }

    async fn test_add_user(&self) {
        use webserver_contracts::user::*;

        let params = AddUserParams::new(User::new("test_user".into(), "test_password".into()));
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::AddUser.to_string(),
            serde_json::to_value(params).unwrap(),
            Some("mngr_test_add_user".into()),
        );

        let response: JsonRpcResponse = self.client.send_request(request).await.unwrap();

        match response.kind() {
            ResponseKind::Success => info!("success response"),
            ResponseKind::Error(e) => error!("{:?}", e),
        }
    }

    async fn test_set_role(&self) {
        let params = SetRoleParams::new(
            User::new("test_user".to_string(), "test_password".to_string()),
            "test_user".to_string(),
            "admin".to_string(),
        );

        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::SetRole.to_string(),
            params,
            Some("mngr_test_set_role".into()),
        );

        let response: JsonRpcResponse = self.client.send_request(request).await.unwrap();

        match response.kind() {
            ResponseKind::Success => info!("success response"),
            ResponseKind::Error(e) => error!("{:?}", e),
        }
    }

    async fn test_delete_user(&self) {
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
