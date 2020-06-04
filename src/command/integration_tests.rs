use crate::IntegrationTestsOpts;
use webserver_client::WebserverClient;
use webserver_contracts::{
    prediction::{AddPredictionParams, SearchPredictionsParams, SearchPredictionsResult},
    user::{
        AddUserParams, DeleteUserParams, DeleteUserResult, User, ValidateUserParams,
        ValidateUserResult,
    },
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

    pub async fn run_tests(&self) -> Result<(), String> {
        info!("Adding user '{}'...", self.test_user.username());
        self.add_user().await?;
        info!("Validating user '{}'...", self.test_user.username());
        self.validate_user().await?;
        info!("Adding a prediction by 'test_user'...");
        self.add_prediction().await?;
        info!("Searching for predictions by 'test_user'...");
        self.search_prediction_without_user().await?;
        info!("Deleting user 'test_user'...");
        self.delete_user().await?;

        Ok(())
    }

    async fn add_user(&self) -> Result<(), String> {
        let params = AddUserParams::new(self.test_user.clone());
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::AddUser.to_string(),
            serde_json::to_value(params).unwrap(),
            Some("mngr_add_user".into()),
        );

        let response = self
            .client
            .send_request(request)
            .await
            .map_err(|e| format!("{:?}", e))?;

        match response.kind() {
            ResponseKind::Success(_) => Ok(()),
            ResponseKind::Error(e) => Err(format!("{:?}", e)),
        }
    }

    async fn validate_user(&self) -> Result<(), String> {
        let params = ValidateUserParams::new(self.test_user.clone());
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::ValidateUser.to_string(),
            params,
            Some("mngr_validate_user".into()),
        );

        let response = self
            .client
            .send_request(request)
            .await
            .map_err(|e| format!("{:?}", e))?;

        let result: ValidateUserResult = match response.kind() {
            ResponseKind::Success(_) => response.result_as().unwrap(),
            ResponseKind::Error(e) => return Err(format!("{:?}", e)),
        };

        if result.valid() {
            trace!("'{}' is valid", self.test_user.username());
            Ok(())
        } else {
            Err(format!("'{}' is not valid", self.test_user.username()))
        }
    }

    async fn add_prediction(&self) -> Result<(), String> {
        let params = AddPredictionParams::new("Test prediction".into(), self.test_user.clone());
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::AddPrediction.to_string(),
            params,
            Some("mngr_add_prediction".into()),
        );

        let response = self.client.send_request(request).await.unwrap();

        match response.kind() {
            ResponseKind::Success(_) => Ok(()),
            ResponseKind::Error(e) => Err(format!("{:?}", e)),
        }
    }

    async fn search_prediction_without_user(&self) -> Result<(), String> {
        let params = SearchPredictionsParams::new(self.test_user.username().to_owned(), None);
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::SearchPredictions.to_string(),
            params,
            Some("mngr_search_prediction_without_user".into()),
        );

        let response: JsonRpcResponse = self.client.send_request(request).await.unwrap();

        let result: SearchPredictionsResult = match response.kind() {
            ResponseKind::Success(_) => response.result_as().unwrap(),
            ResponseKind::Error(e) => return Err(format!("{:?}", e)),
        };

        if result.predictions().len() != 1 {
            return Err(format!(
                "should be exactly 1 prediction returned, was {}",
                result.predictions().len()
            ));
        }

        trace!(
            "found predictions by '{}': {:#?}",
            self.test_user.username(),
            result.predictions()
        );

        Ok(())
    }

    async fn delete_user(&self) -> Result<(), String> {
        let params = DeleteUserParams::new(
            User::new("test_user".into(), "test_password".into()),
            "test_user".into(),
        );
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::DeleteUser.to_string(),
            params,
            Some("mngr_delete_user".into()),
        );

        let response: JsonRpcResponse = self.client.send_request(request).await.unwrap();

        let result: DeleteUserResult = match response.kind() {
            ResponseKind::Success(_) => response.result_as().unwrap(),
            ResponseKind::Error(e) => return Err(format!("{:?}", e)),
        };

        if !result.success() {
            Err(format!("failed to delete user"))
        } else if result.deleted_predictions() != 1 {
            Err(format!(
                "should have only deleted 1 prediction, deleted {}",
                result.deleted_predictions()
            ))
        } else {
            trace!("deleted {} predictions", result.deleted_predictions());
            Ok(())
        }
    }
}
