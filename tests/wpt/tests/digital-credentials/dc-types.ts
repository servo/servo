export type GetProtocol = "default" | "openid4vp";
export type CreateProtocol = "default" | "openid4vci";

export type CredentialMediationRequirement =
  | "conditional"
  | "optional"
  | "required"
  | "silent";

/**
 * @see https://wicg.github.io/digital-credentials/#dom-digitalcredentialrequest
 */
export interface DigitalCredentialGetRequest {
  protocol: string;
  data: object;
}

/**
 * @see https://wicg.github.io/digital-credentials/#dom-digitalcredentialrequestoptions
 */
export interface DigitalCredentialRequestOptions {
  /**
   * The list of credential requests.
   */
  requests: DigitalCredentialGetRequest[] | any;
}

/**
 * @see https://wicg.github.io/digital-credentials/#extensions-to-credentialrequestoptions
 */
export interface CredentialRequestOptions {
  digital: DigitalCredentialRequestOptions;
  mediation: CredentialMediationRequirement;
}

export interface DigitalCredentialCreateRequest {
  protocol: string;
  data: object;
}

export interface DigitalCredentialCreationOptions {
  /**
   * The list of credential requests.
   */
  requests: DigitalCredentialCreateRequest[] | any;
}

export interface CredentialCreationOptions {
  digital: DigitalCredentialCreationOptions;
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
