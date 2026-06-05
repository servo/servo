use rustenium_bidi_definitions::network::commands::NetworkCommand;

use crate::{error::WebDriverBidiError, handler::Handler, model::NetworkResult};

impl Handler {
    pub(super) async fn handle_network(
        &self,
        cmd: NetworkCommand,
    ) -> Result<NetworkResult, WebDriverBidiError> {
        match cmd {
            NetworkCommand::AddDataCollector(add_data_collector) => {
                self.handle_network_add_data_collector().await
            },
            NetworkCommand::AddIntercept(add_intercept) => {
                self.handle_network_add_intercept().await
            },
            NetworkCommand::ContinueRequest(continue_request) => {
                self.handle_network_continue_request().await
            },
            NetworkCommand::ContinueResponse(continue_response) => {
                self.handle_network_continue_response().await
            },
            NetworkCommand::ContinueWithAuth(continue_with_auth) => {
                self.handle_network_continue_with_auth().await
            },
            NetworkCommand::DisownData(disown_data) => self.handle_network_disown_data().await,
            NetworkCommand::FailRequest(fail_request) => self.handle_network_fail_request().await,
            NetworkCommand::GetData(get_data) => self.handle_network_get_data().await,
            NetworkCommand::ProvideResponse(provide_response) => {
                self.handle_network_provide_response().await
            },
            NetworkCommand::RemoveDataCollector(remove_data_collector) => {
                self.handle_network_remove_data_collector().await
            },
            NetworkCommand::RemoveIntercept(remove_intercept) => {
                self.handle_network_remove_intercept().await
            },
            NetworkCommand::SetCacheBehavior(set_cache_behavior) => {
                self.handle_network_set_cache_behavior().await
            },
            NetworkCommand::SetExtraHeaders(set_extra_headers) => {
                self.handle_network_set_extra_headers().await
            },
        }
    }

    async fn handle_network_add_data_collector(&self) -> Result<NetworkResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_network_add_intercept(&self) -> Result<NetworkResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_network_continue_request(&self) -> Result<NetworkResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_network_continue_response(&self) -> Result<NetworkResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_network_continue_with_auth(&self) -> Result<NetworkResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_network_disown_data(&self) -> Result<NetworkResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_network_fail_request(&self) -> Result<NetworkResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_network_get_data(&self) -> Result<NetworkResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_network_provide_response(&self) -> Result<NetworkResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_network_remove_data_collector(
        &self,
    ) -> Result<NetworkResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_network_remove_intercept(&self) -> Result<NetworkResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_network_set_cache_behavior(&self) -> Result<NetworkResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_network_set_extra_headers(&self) -> Result<NetworkResult, WebDriverBidiError> {
        todo!()
    }
}
