use crate::IntegrationTestsOpts;
use influx::{InfluxClient, Measurement};
use uuid::Uuid;
use webserver_client::WebserverClient;
use webserver_contracts::{
    list::{AddListItemParams, AddListItemResult},
    prediction::{AddPredictionParams, SearchPredictionsParams, SearchPredictionsResult},
    user::{
        AddUserParams, AddUserResult, DeleteUserParams, DeleteUserResult, User, ValidateUserParams,
        ValidateUserResult,
    },
    JsonRpcRequest, JsonRpcRequestBuilder, JsonRpcResponse, JsonRpcVersion, Method, ResponseKind,
};

pub struct IntegrationTests {
    opts: IntegrationTestsOpts,
    client: WebserverClient,
    influx_client: InfluxClient,
    test_user: User,
    other_user: User,
}

impl IntegrationTests {
    pub fn new(opts: IntegrationTestsOpts) -> Self {
        let client = WebserverClient::new()
            .with_url(opts.url.clone())
            .build()
            .unwrap();
        let influx_client = InfluxClient::builder(
            opts.influx_url.clone(),
            opts.influx_key.clone(),
            opts.influx_org.clone(),
        )
        .build()
        .unwrap();
        let test_user = User::new("test_user".to_string(), "test_password".to_string());
        let other_user = User::new("other_user".to_string(), "other_password".to_string());
        Self {
            opts,
            client,
            influx_client,
            test_user,
            other_user,
        }
    }

    pub async fn run_tests(&self) -> Result<(), String> {
        let timer = std::time::Instant::now();
        info!("adding user '{}'...", self.test_user.username);
        self.add_user(self.test_user.clone()).await?;
        info!("adding user '{}'...", self.other_user.username);
        self.add_user(self.other_user.clone()).await?;

        info!("validating user '{}'...", self.test_user.username);
        self.validate_user(&self.test_user).await?;
        info!("validating user '{}'...", self.other_user.username);
        self.validate_user(&self.other_user).await.unwrap();
        info!(
            "validating user '{}' with wrong password",
            self.test_user.username
        );
        self.validate_user(&User::new(
            self.test_user.username.to_owned(),
            "wrong_password".into(),
        ))
        .await
        .unwrap_err();

        info!("adding a prediction by '{}'...", self.test_user.username);
        self.add_prediction_by_test_user().await?;
        info!(
            "searching for predictions by '{}'...",
            self.test_user.username
        );
        self.search_prediction_by_test_user().await?;
        info!(
            "searching for predictions by '{}' as '{}'...",
            self.test_user.username, self.test_user.username
        );
        self.search_prediction_by_test_user_as_test_user().await?;
        info!("deleting user '{}'...", self.test_user.username);
        self.delete_test_user().await?;
        info!("deleting user '{}'...", self.other_user.username);
        self.delete_other_user().await?;

        self.influx_client
            .send_batch(
                "mngr",
                &vec![Measurement::builder("integration_test_execution".into())
                    .with_tag("success".into(), "true".into())
                    .with_field_u128("duration_ms".into(), timer.elapsed().as_millis())
                    .build()
                    .unwrap()],
            )
            .await;
        Ok(())
    }

    async fn add_user(&self, user: User) -> Result<(), String> {
        let username = user.username.to_owned();
        let params = AddUserParams::new(user);
        let request = JsonRpcRequestBuilder::new()
            .with_method(Method::AddUser.to_string())
            .with_id("add_user".into())
            .with_params(params)
            .build()?;

        let response = self
            .client
            .send_request(request)
            .await
            .map_err(|e| format!("{:?}", e))?;

        let result: AddUserResult = match response.kind() {
            ResponseKind::Success(_) => response.result_as().unwrap().unwrap(),
            ResponseKind::Error(e) => return Err(format!("{:?}", e)),
        };

        if result.success {
            Ok(())
        } else {
            Err(format!("failed to add user: '{}'", username))
        }
    }

    async fn validate_user(&self, user: &User) -> Result<(), String> {
        let params = ValidateUserParams::new(user.clone());
        let request = JsonRpcRequestBuilder::new()
            .with_method(Method::ValidateUser.to_string())
            .with_params(params)
            .with_id("validate_user".into())
            .build()?;

        let response = self
            .client
            .send_request(request)
            .await
            .map_err(|e| format!("{:?}", e))?;
        let result: ValidateUserResult = match response.kind() {
            ResponseKind::Success(_) => response.result_as().unwrap().unwrap(),
            ResponseKind::Error(e) => return Err(format!("{:?}", e)),
        };

        if result.valid {
            Ok(())
        } else {
            Err(format!("user '{}' is not valid", user.username))
        }
    }

    async fn add_prediction_by_test_user(&self) -> Result<(), String> {
        let params = AddPredictionParams::new("Test prediction".into(), self.test_user.clone());
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::AddPrediction.to_string(),
            params,
            Some("add_prediction_by_test_user".into()),
        );

        let response = self.client.send_request(request).await.unwrap();

        match response.kind() {
            ResponseKind::Success(_) => Ok(()),
            ResponseKind::Error(e) => Err(format!("{:?}", e)),
        }
    }

    async fn add_prediction_by_other_user(&self) -> Result<(), String> {
        let params = AddPredictionParams::new("Other prediction".into(), self.other_user.clone());
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::AddPrediction.to_string(),
            params,
            Some("add_prediction_by_other_user".into()),
        );

        let response = self.client.send_request(request).await.unwrap();

        match response.kind() {
            ResponseKind::Success(_) => Ok(()),
            ResponseKind::Error(e) => Err(format!("{:?}", e)),
        }
    }

    async fn search_prediction_by_test_user(&self) -> Result<(), String> {
        let params = SearchPredictionsParams::new(self.test_user.username.to_owned(), None);
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::SearchPredictions.to_string(),
            params,
            Some("search_prediction_by_test_user".into()),
        );

        let response: JsonRpcResponse = self.client.send_request(request).await.unwrap();

        let result: SearchPredictionsResult = match response.kind() {
            ResponseKind::Success(_) => response.result_as().unwrap().unwrap(),
            ResponseKind::Error(e) => return Err(format!("{:?}", e)),
        };

        if result.predictions.len() != 1 {
            return Err(format!(
                "should be exactly 1 prediction returned, was {}",
                result.predictions.len()
            ));
        }

        if result.predictions[0].id.is_some() {
            return Err(format!(
                "should not get id in response when no user is provided"
            ));
        }

        trace!(
            "found predictions by '{}': {:#?}",
            self.test_user.username,
            result.predictions
        );

        Ok(())
    }

    async fn search_prediction_by_test_user_as_test_user(&self) -> Result<(), String> {
        let params = SearchPredictionsParams::new(
            self.test_user.username.to_owned(),
            Some(self.test_user.clone()),
        );
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::SearchPredictions.to_string(),
            params,
            Some("search_prediction_by_test_user_as_test_user".into()),
        );

        let response: JsonRpcResponse = self.client.send_request(request).await.unwrap();

        let result: SearchPredictionsResult = match response.kind() {
            ResponseKind::Success(_) => response.result_as().unwrap().unwrap(),
            ResponseKind::Error(e) => return Err(format!("{:?}", e)),
        };

        if result.predictions.len() != 1 {
            return Err(format!(
                "should be exactly 1 prediction returned, was {}",
                result.predictions.len()
            ));
        }

        if result.predictions[0].id.is_none() {
            return Err(format!(
                "should get id in response when same user is provided"
            ));
        }

        trace!(
            "found predictions by '{}': {:#?}",
            self.test_user.username,
            result.predictions
        );

        Ok(())
    }

    async fn delete_test_user(&self) -> Result<(), String> {
        let params =
            DeleteUserParams::new(self.test_user.clone(), self.test_user.username.to_owned());
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::DeleteUser.to_string(),
            params,
            Some("delete_test_user".into()),
        );

        let response: JsonRpcResponse = self.client.send_request(request).await.unwrap();

        let result: DeleteUserResult = match response.kind() {
            ResponseKind::Success(_) => response.result_as().unwrap().unwrap(),
            ResponseKind::Error(e) => return Err(format!("{:?}", e)),
        };

        if !result.success {
            Err(format!("failed to delete user"))
        } else if result.deleted_predictions != 1 {
            Err(format!(
                "should have only deleted 1 prediction, deleted {}",
                result.deleted_predictions
            ))
        } else {
            trace!("deleted {} predictions", result.deleted_predictions);
            Ok(())
        }
    }

    async fn delete_other_user(&self) -> Result<(), String> {
        let params =
            DeleteUserParams::new(self.other_user.clone(), self.other_user.username.to_owned());
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::DeleteUser.to_string(),
            params,
            Some("delete_other_user".into()),
        );

        let response: JsonRpcResponse = self.client.send_request(request).await.unwrap();

        let result: DeleteUserResult = match response.kind() {
            ResponseKind::Success(_) => response.result_as().unwrap().unwrap(),
            ResponseKind::Error(e) => return Err(format!("{:?}", e)),
        };

        if !result.success {
            Err(format!("failed to delete user"))
        } else if result.deleted_predictions != 0 {
            Err(format!(
                "should deleted 0 predictions, deleted {}",
                result.deleted_predictions
            ))
        } else {
            trace!("deleted {} predictions", result.deleted_predictions);
            Ok(())
        }
    }

    async fn add_test_list_item(&self) -> Result<(), String> {
        let params = AddListItemParams::new(
            None,
            String::from("test_list"),
            String::from("test_item"),
            None,
            None,
        );
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::AddListItem.to_string(),
            params,
            Some(Uuid::new_v4().to_string()),
        );

        let response: JsonRpcResponse = self.client.send_request(request).await.unwrap();

        let _result: AddListItemResult = match response.kind() {
            ResponseKind::Success(_) => response.result_as().unwrap().unwrap(),
            ResponseKind::Error(e) => return Err(format!("{:?}", e)),
        };

        Ok(())
    }
}
