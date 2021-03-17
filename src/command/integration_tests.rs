use crate::IntegrationTestsOpts;
use uuid::Uuid;
use webserver_client::WebserverClient;
use webserver_contracts::{JsonRpcRequest, JsonRpcVersion, Method, list::{AddListItemParams, AddListItemResult, DeleteListItemParams, DeleteListItemResult}};

pub struct IntegrationTests {
    opts: IntegrationTestsOpts,
    client: WebserverClient,
}

impl IntegrationTests {
    pub fn new(opts: IntegrationTestsOpts) -> Self {
        let client = WebserverClient::new(
            opts.url.clone(),
            opts.key_name.clone(),
            opts.key_value.clone(),
        )
        .build()
        .unwrap();
        Self { opts, client }
    }

    pub async fn run_tests(&self) -> Result<(), String> {
        self.add_and_delete_list_item().await?;

        Ok(())
    }

    async fn add_and_delete_list_item(&self) -> Result<(), String> {
        let params = AddListItemParams::new(
            None,
            "test_list_type".to_string(),
            "Test item".to_string(),
            None,
            None,
        );
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::AddListItem.to_string(),
            params,
            Some(Uuid::new_v4().to_string()),
        );

        let response = self
            .client
            .send_request(request)
            .await
            .map_err(|e| e.to_string())?;

        let result: AddListItemResult = response.result_as().unwrap().unwrap();

        let params = DeleteListItemParams::new(result.id.unwrap());
        let request = JsonRpcRequest::new(
            JsonRpcVersion::Two,
            Method::DeleteListItem.to_string(),
            params,
            Some(Uuid::new_v4().to_string()),
        );
        let response = self
            .client
            .send_request(request)
            .await
            .map_err(|e| e.to_string())?;

        let result: DeleteListItemResult = response.result_as().unwrap().unwrap();

        Ok(())
    }
}
