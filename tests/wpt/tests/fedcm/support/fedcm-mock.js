import { RequestTokenStatus, LogoutRpsStatus, DisconnectStatus, FederatedAuthRequest, FederatedAuthRequestReceiver } from '/gen/third_party/blink/public/mojom/webid/federated_auth_request.mojom.m.js';

function toMojoTokenStatus(status) {
  return RequestTokenStatus["k" + status];
}

// A mock service for responding to federated auth requests.
export class MockFederatedAuthRequest {
  constructor() {
    this.receiver_ = new FederatedAuthRequestReceiver(this);
    this.interceptor_ = new MojoInterfaceInterceptor(FederatedAuthRequest.$interfaceName);
    this.interceptor_.oninterfacerequest = e => {
        this.receiver_.$.bindHandle(e.handle);
    }
    this.interceptor_.start();
    this.token_ = null;
    this.selected_identity_provider_config_url_ = null;
    this.status_ = RequestTokenStatus.kError;
    this.logoutRpsStatus_ = LogoutRpsStatus.kError;
    this.disconnectStatus_ = DisconnectStatus.kError;
    this.returnPending_ = false;
    this.pendingPromiseResolve_ = null;
  }

  // Causes the subsequent `navigator.credentials.get()` to resolve with the token.
  returnToken(selected_identity_provider_config_url, token) {
    this.status_ = RequestTokenStatus.kSuccess;
    this.selected_identity_provider_config_url_ = selected_identity_provider_config_url;
    this.token_ = token;
    this.returnPending_ = false;
  }

  // Causes the subsequent `navigator.credentials.get()` to reject with the error.
  returnError(error) {
    if (error == "Success")
      throw new Error("Success is not a valid error");
    this.status_ = toMojoTokenStatus(error);
    this.selected_identity_provider_config_url_ = null;
    this.token_ = null;
    this.returnPending_ = false;
  }

  // Causes the subsequent `navigator.credentials.get()` to return a pending promise
  // that can be cancelled using `cancelTokenRequest()`.
  returnPendingPromise() {
    this.returnPending_ = true;
  }

  logoutRpsReturn(status) {
    let validated = LogoutRpsStatus[status];
    if (validated === undefined)
      throw new Error("Invalid status: " + status);
    this.logoutRpsStatus_ = validated;
  }

  // Causes the subsequent `FederatedCredential.disconnect` to reject with this
  // status.
  disconnectReturn(status) {
    let validated = DisconnectStatus[status];
    if (validated === undefined)
      throw new Error("Invalid status: " + status);
    this.disconnectStatus_ = validated;
  }

  // Implements
  //   RequestToken(array<IdentityProviderGetParameters> idp_get_params) =>
  //                    (RequestTokenStatus status,
  //                      url.mojom.Url? selected_identity_provider_config_url,
  //                      string? token);
  async requestToken(idp_get_params) {
    if (this.returnPending_) {
      this.pendingPromise_ = new Promise((resolve, reject) => {
        this.pendingPromiseResolve_ = resolve;
      });
      return this.pendingPromise_;
    }
    return Promise.resolve({
      status: this.status_,
      selected_identity_provider_config_url: this.selected_identity_provider_config_url_,
      token: this.token_
    });
  }

  async cancelTokenRequest() {
    this.pendingPromiseResolve_({
      status: toMojoTokenStatus("ErrorCanceled"),
      selected_identity_provider_config_url: null,
      token: null
    });
    this.pendingPromiseResolve_ = null;
  }

  // Implements
  //   RequestUserInfo(IdentityProviderGetParameters idp_get_param) =>
  //                    (RequestUserInfoStatus status, array<IdentityUserInfo>? user_info);
  async requestUserInfo(idp_get_param) {
    return Promise.resolve({
      status: "",
      user_info: ""
    });
  }

  async logoutRps(logout_endpoints) {
    return Promise.resolve({
      status: this.logoutRpsStatus_
    });
  }

  async disconnect(provider, client_id, account_id) {
    return Promise.resolve({
      status: this.disconnectStatus_
    });
  }

  async setIdpSigninStatus(origin, status) {
  }

  async registerIdP(configURL) {
  }

  async unregisterIdP(configURL) {
  }

  async resolveTokenRequest(token) {
  }

  async closeModalDialogView() {
  }

  async preventSilentAccess() {
  }

  async reset() {
    this.token_ = null;
    this.selected_identity_provider_config_url_ = null;
    this.status_ = RequestTokenStatus.kError;
    this.logoutRpsStatus_ = LogoutRpsStatus.kError;
    this.disconnectStatus_ = DisconnectStatus.kError;
    this.receiver_.$.close();
    this.interceptor_.stop();

    // Clean up and reset mock stubs asynchronously, so that the blink side
    // closes its proxies and notifies JS sensor objects before new test is
    // started.
    await new Promise(resolve => { step_timeout(resolve, 0); });
  }
}
