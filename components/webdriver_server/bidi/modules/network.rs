use std::rc::Rc;

use webdriver_traits::bidi::{
    NetworkCommand, NetworkResult,
    network::{
        AddDataCollectorParameters, AddDataCollectorResult, AddInterceptParameters,
        AddInterceptResult, ContinueRequestParameters, ContinueRequestResult,
        ContinueResponseParameters, ContinueResponseResult, ContinueWithAuthParameters,
        ContinueWithAuthResult, DisownDataParameters, DisownDataResult, FailRequestParameters,
        FailRequestResult, GetDataParameters, GetDataResult, ProvideResponseParameters,
        ProvideResponseResult, RemoveDataCollectorParameters, RemoveDataCollectorResult,
        RemoveInterceptParameters, RemoveInterceptResult, SetCacheBehaviorParameters,
        SetCacheBehaviorResult, SetExtraHeadersParameters, SetExtraHeadersResult,
    },
};

use crate::bidi::{error::BidiResult, remote_end::RemoteEnd};

impl RemoteEnd {
    pub(crate) async fn handle_network_command(
        self: Rc<Self>,
        command: NetworkCommand,
    ) -> BidiResult<NetworkResult> {
        match command {
            NetworkCommand::AddDataCollector(cmd) => self
                .handle_network_add_data_collector(cmd.params)
                .await
                .map(NetworkResult::AddDataCollectorResult),
            NetworkCommand::AddIntercept(cmd) => self
                .handle_network_add_intercept(cmd.params)
                .await
                .map(NetworkResult::AddInterceptResult),
            NetworkCommand::ContinueRequest(cmd) => self
                .handle_network_continue_request(cmd.params)
                .await
                .map(NetworkResult::ContinueRequestResult),
            NetworkCommand::ContinueResponse(cmd) => self
                .handle_network_continue_response(cmd.params)
                .await
                .map(NetworkResult::ContinueResponseResult),
            NetworkCommand::ContinueWithAuth(cmd) => self
                .handle_network_continue_with_auth(cmd.params)
                .await
                .map(NetworkResult::ContinueWithAuthResult),
            NetworkCommand::DisownData(cmd) => self
                .handle_network_disown_data(cmd.params)
                .await
                .map(NetworkResult::DisownDataResult),
            NetworkCommand::FailRequest(cmd) => self
                .handle_network_fail_request(cmd.params)
                .await
                .map(NetworkResult::FailRequestResult),
            NetworkCommand::GetData(cmd) => self
                .handle_network_get_data(cmd.params)
                .await
                .map(NetworkResult::GetDataResult),
            NetworkCommand::ProvideResponse(cmd) => self
                .handle_network_provide_response(cmd.params)
                .await
                .map(NetworkResult::ProvideResponseResult),
            NetworkCommand::RemoveDataCollector(cmd) => self
                .handle_network_remove_data_collector(cmd.params)
                .await
                .map(NetworkResult::RemoveDataCollectorResult),
            NetworkCommand::RemoveIntercept(cmd) => self
                .handle_network_remove_intercept(cmd.params)
                .await
                .map(NetworkResult::RemoveInterceptResult),
            NetworkCommand::SetCacheBehavior(cmd) => self
                .handle_network_set_cache_behavior(cmd.params)
                .await
                .map(NetworkResult::SetCacheBehaviorResult),
            NetworkCommand::SetExtraHeaders(cmd) => self
                .handle_network_set_extra_headers(cmd.params)
                .await
                .map(NetworkResult::SetExtraHeadersResult),
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-addDataCollector>
    async fn handle_network_add_data_collector(
        self: Rc<Self>,
        _: AddDataCollectorParameters,
    ) -> BidiResult<AddDataCollectorResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-addIntercept>
    async fn handle_network_add_intercept(
        self: Rc<Self>,
        _: AddInterceptParameters,
    ) -> BidiResult<AddInterceptResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-continueRequest>
    async fn handle_network_continue_request(
        self: Rc<Self>,
        _: ContinueRequestParameters,
    ) -> BidiResult<ContinueRequestResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-continueResponse>
    async fn handle_network_continue_response(
        self: Rc<Self>,
        _: ContinueResponseParameters,
    ) -> BidiResult<ContinueResponseResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-continueWithAuth>
    async fn handle_network_continue_with_auth(
        self: Rc<Self>,
        _: ContinueWithAuthParameters,
    ) -> BidiResult<ContinueWithAuthResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-disownData>
    async fn handle_network_disown_data(
        self: Rc<Self>,
        _: DisownDataParameters,
    ) -> BidiResult<DisownDataResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-failRequest>
    async fn handle_network_fail_request(
        self: Rc<Self>,
        _: FailRequestParameters,
    ) -> BidiResult<FailRequestResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-getData>
    async fn handle_network_get_data(
        self: Rc<Self>,
        _: GetDataParameters,
    ) -> BidiResult<GetDataResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-provideResponse>
    async fn handle_network_provide_response(
        self: Rc<Self>,
        _: ProvideResponseParameters,
    ) -> BidiResult<ProvideResponseResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-removeDataCollector>
    async fn handle_network_remove_data_collector(
        self: Rc<Self>,
        _: RemoveDataCollectorParameters,
    ) -> BidiResult<RemoveDataCollectorResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-removeIntercept>
    async fn handle_network_remove_intercept(
        self: Rc<Self>,
        _: RemoveInterceptParameters,
    ) -> BidiResult<RemoveInterceptResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-setCacheBehavior>
    async fn handle_network_set_cache_behavior(
        self: Rc<Self>,
        _: SetCacheBehaviorParameters,
    ) -> BidiResult<SetCacheBehaviorResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-setExtraHeaders>
    async fn handle_network_set_extra_headers(
        self: Rc<Self>,
        _: SetExtraHeadersParameters,
    ) -> BidiResult<SetExtraHeadersResult> {
        todo!()
    }
}
