export type ProviderType = "default" | "openid4vp";
export type CredentialMediationRequirement =
  | "conditional"
  | "optional"
  | "required"
  | "silent";

/**
 * @see https://wicg.github.io/digital-credentials/#dom-identityrequestprovider
 */
export interface IdentityRequestProvider {
  protocol: string;
  request: object;
}

/**
 * @see https://wicg.github.io/digital-credentials/#dom-digitalcredentialrequestoptions
 */
export interface DigitalCredentialRequestOptions {
  /**
   * The list of identity request providers
   */
  providers: IdentityRequestProvider[] | any;
}

/**
 * @see https://wicg.github.io/digital-credentials/#extensions-to-credentialrequestoptions-dictionary
 */
export interface CredentialRequestOptions {
  /**
   * The digital credential request options.
   */
  digital: DigitalCredentialRequestOptions;
  mediation: CredentialMediationRequirement;
}

/**
 * The actions that can be performed on the API via the iframe.
 */
export type IframeActionType =
  | "create"
  | "get"
  | "ping"
  | "preventSilentAccess";

/**
 * If present, when the abort controller should be aborted
 * relative the invocation of the API.
 */
export type AbortType = "before" | "after";

export interface EventData {
  /**
   * Action to perform on the API.
   */
  action: IframeActionType;
  /**
   * If the action should be aborted, and when.
   */
  abort?: AbortType;
  /**
   * The options to pass to the API.
   */
  options?: object;
  /**
   * If the API needs to blessed before the action is performed.
   */
  needsUserActivation?: boolean;
}

export interface SendMessageData {
  action: IframeActionType;
  options?: CredentialRequestOptions;
}
