use crate::IntegrationTestsOpts;
use client::WebserverClient;
use contracts::{
    list::{add_list_item, delete_list_item},
    JsonRpcRequest, Method,
};
use uuid::Uuid;

pub struct IntegrationTests {
    opts: IntegrationTestsOpts,
    client: WebserverClient,
}

impl IntegrationTests {
    pub fn new(opts: IntegrationTestsOpts) -> Self {
        let client = WebserverClient::builder(
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
        let params =
            add_list_item::Params::new(None, "test_list_type".to_string(), "Test item".to_string())
                .unwrap();
        let request = JsonRpcRequest::new(
            Method::AddListItem.to_string(),
            params,
            Some(Uuid::new_v4().to_string()),
        );

        let response = self
            .client
            .send_request(&request)
            .await
            .map_err(|e| e.to_string())?;

        let result: add_list_item::MethodResult = response.unwrap().result_as().unwrap().unwrap();

        let params = delete_list_item::Params::new(result.id.unwrap());
        let request = JsonRpcRequest::new(
            Method::DeleteListItem.to_string(),
            params,
            Some(Uuid::new_v4().to_string()),
        );
        let response = self
            .client
            .send_request(&request)
            .await
            .map_err(|e| e.to_string())?;

        let result: delete_list_item::MethodResult =
            response.unwrap().result_as().unwrap().unwrap();

        Ok(())
    }
}
