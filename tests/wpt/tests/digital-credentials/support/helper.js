// @ts-check
// Import the types from the TypeScript file
/**
 * @typedef {import('../dc-types').GetProtocol} GetProtocol
 * @typedef {import('../dc-types').DigitalCredentialGetRequest} DigitalCredentialGetRequest
 * @typedef {import('../dc-types').CredentialRequestOptions} CredentialRequestOptions
 * @typedef {import('../dc-types').CreateProtocol} CreateProtocol
 * @typedef {import('../dc-types').DigitalCredentialCreateRequest} DigitalCredentialCreateRequest
 * @typedef {import('../dc-types').CredentialCreationOptions} CredentialCreationOptions
 * @typedef {import('../dc-types').SendMessageData} SendMessageData
 * @typedef {import('../dc-types').MakeGetOptionsConfig} MakeGetOptionsConfig
 * @typedef {import('../dc-types').MakeCreateOptionsConfig} MakeCreateOptionsConfig
 * @typedef {import('../dc-types').CredentialMediationRequirement} CredentialMediationRequirement
 * @typedef {import('../dc-types').MobileDocumentRequest} MobileDocumentRequest
 * @typedef {GetProtocol | CreateProtocol} Protocol
 */

/** @type {Record<Protocol, object | MobileDocumentRequest>} */
const CANONICAL_REQUEST_OBJECTS = {
  openid4vci: {
    /* Canonical object coming soon */
  },
  "openid4vp-v1-unsigned": {
    /* Canonical object coming soon */
  },
  "openid4vp-v1-signed": {
    /* Canonical object coming soon */
  },
  "openid4vp-v1-multisigned": {
    /* Canonical object coming soon */
  },
  /** @type MobileDocumentRequest **/
  "org-iso-mdoc": {
    deviceRequest:
      "omd2ZXJzaW9uYzEuMGtkb2NSZXF1ZXN0c4GhbGl0ZW1zUmVxdWVzdNgYWIKiZ2RvY1R5cGV1b3JnLmlzby4xODAxMy41LjEubURMam5hbWVTcGFjZXOhcW9yZy5pc28uMTgwMTMuNS4x9pWthZ2Vfb3Zlcl8yMfRqZ2l2ZW5fbmFtZfRrZmFtaWx5X25hbWX0cmRyaXZpbmdfcHJpdmlsZWdlc_RocG9ydHJhaXT0",
    encryptionInfo:
      "gmVkY2FwaaJlbm9uY2VYICBetSsDkKlE_G9JSIHwPzr3ctt6Ol9GgmCH8iGdGQNJcnJlY2lwaWVudFB1YmxpY0tleaQBAiABIVggKKm1iPeuOb9bDJeeJEL4QldYlWvY7F_K8eZkmYdS9PwiWCCm9PLEmosiE_ildsE11lqq4kDkjhfQUKPpbX-Hm1ZSLg",
  },
};

/**
 * Internal helper to create final options from a list of requests.
 *
 * @template {DigitalCredentialGetRequest[] | DigitalCredentialCreateRequest[]} TRequests
 * @template {CredentialRequestOptions | CredentialCreationOptions} TOptions
 * @param {TRequests} requests
 * @param {CredentialMediationRequirement} [mediation]
 * @param {AbortSignal} [signal]
 * @returns {TOptions}
 */
function makeOptionsFromRequests(requests, mediation, signal) {
  /** @type {TOptions} */
  const options = /** @type {TOptions} */ ({ digital: { requests } });

  if (mediation) {
    options.mediation = mediation;
  }

  if (signal) {
    options.signal = signal;
  }

  return options;
}

/**
 * Build requests from protocols, using canonical data for each protocol.
 * For create operations with explicit data, uses that data for all protocols.
 *
 * @template Req
 * @param {Protocol[]} protocols
 * @param {Record<string, (data?: MobileDocumentRequest | object) => Req>} mapping
 * @param {MobileDocumentRequest | object} [explicitData] - Explicit data for create operations
 * @returns {Req[]}
 * @throws {Error} If an unknown protocol string is encountered.
 */
function buildRequestsFromProtocols(protocols, mapping, explicitData) {
  return protocols.map((protocol) => {
    if (!(protocol in mapping)) {
      throw new Error(`Unknown request type within array: ${protocol}`);
    }
    // Use explicit data if provided (for create with data), otherwise canonical data
    return mapping[protocol](explicitData);
  });
}

/** @type {{
 *   get: Record<GetProtocol, (data?: MobileDocumentRequest | object) => DigitalCredentialGetRequest>;
 *   create: Record<CreateProtocol, (data?: object) => DigitalCredentialCreateRequest>;
 * }} */
const allMappings = {
  get: {
    "org-iso-mdoc": (
      data = { ...CANONICAL_REQUEST_OBJECTS["org-iso-mdoc"] },
    ) => {
      return { protocol: "org-iso-mdoc", data };
    },
    "openid4vp-v1-unsigned": (
      data = { ...CANONICAL_REQUEST_OBJECTS["openid4vp-v1-unsigned"] },
    ) => {
      return { protocol: "openid4vp-v1-unsigned", data };
    },
    "openid4vp-v1-signed": (
      data = { ...CANONICAL_REQUEST_OBJECTS["openid4vp-v1-signed"] },
    ) => {
      return { protocol: "openid4vp-v1-signed", data };
    },
    "openid4vp-v1-multisigned": (
      data = { ...CANONICAL_REQUEST_OBJECTS["openid4vp-v1-multisigned"] },
    ) => {
      return { protocol: "openid4vp-v1-multisigned", data };
    },
  },
  create: {
    "openid4vci": (data = { ...CANONICAL_REQUEST_OBJECTS["openid4vci"] }) => {
      return { protocol: "openid4vci", data };
    },
  },
};

/**
 * Generic helper to create credential options from config with protocol already set.
 * @template {MakeGetOptionsConfig | MakeCreateOptionsConfig} TConfig
 * @template {DigitalCredentialGetRequest | DigitalCredentialCreateRequest} TRequest
 * @template {CredentialRequestOptions | CredentialCreationOptions} TOptions
 * @param {TConfig} config - Configuration options with protocol already defaulted
 * @param {Record<string, (data?: MobileDocumentRequest | object) => TRequest>} mapping - Protocol to request mapping
 * @returns {TOptions}
 */
function makeCredentialOptionsFromConfig(config, mapping) {
  const { protocol, requests = [], data, mediation, signal } = config;

  // Validate that we have either a protocol or requests
  if (!protocol && !requests?.length) {
    throw new Error("No protocol. Can't make options.");
  }

  /** @type {TRequest[]} */
  const  allRequests = [];

  allRequests.push(.../** @type {TRequest[]} */ (requests));

  if (protocol) {
    const protocolArray = Array.isArray(protocol) ? protocol : [protocol];
    const protocolRequests = buildRequestsFromProtocols(protocolArray, mapping, data);
    allRequests.push(...protocolRequests);
  }

  return /** @type {TOptions} */ (makeOptionsFromRequests(allRequests, mediation, signal));
}

/**
 * Creates options for getting credentials.
 * @export
 * @param {MakeGetOptionsConfig} [config={}] - Configuration options
 * @returns {CredentialRequestOptions}
 */
export function makeGetOptions(config = {}) {
  /** @type {MakeGetOptionsConfig} */
  const configWithDefaults = {
    protocol: ["openid4vp-v1-unsigned", "org-iso-mdoc"],
    ...config,
  };

  return /** @type {CredentialRequestOptions} */ (
    makeCredentialOptionsFromConfig(configWithDefaults, allMappings.get)
  );
}

/**
 * Creates options for creating credentials.
 * @export
 * @param {MakeCreateOptionsConfig} [config={}] - Configuration options
 * @returns {CredentialCreationOptions}
 */
export function makeCreateOptions(config = {}) {
  /** @type {MakeCreateOptionsConfig} */
  const configWithDefaults = {
    protocol: "openid4vci",
    ...config,
  };

  return /** @type {CredentialCreationOptions} */ (
    makeCredentialOptionsFromConfig(configWithDefaults, allMappings.create)
  );
}

/**
 * Sends a message to an iframe and return the response.
 *
 * @param {HTMLIFrameElement} iframe - The iframe element to send the message to.
 * @param {SendMessageData} data - The data to be sent to the iframe.
 * @returns {Promise<any>} - A promise that resolves with the response from the iframe.
 */
export function sendMessage(iframe, data) {
  return new Promise((resolve, reject) => {
    if (!iframe.contentWindow) {
      reject(
        new Error(
          "iframe.contentWindow is undefined, cannot send message (something is wrong with the test that called this).",
        ),
      );
      return;
    }
    window.addEventListener("message", function messageListener(event) {
      if (event.source === iframe.contentWindow) {
        window.removeEventListener("message", messageListener);
        resolve(event.data);
      }
    });
    iframe.contentWindow.postMessage(data, "*");
  });
}

/**
 * Load an iframe with the specified URL and wait for it to load.
 *
 * @param {HTMLIFrameElement} iframe
 * @param {string|URL} url
 * @returns {Promise<void>}
 */
export function loadIframe(iframe, url) {
  return new Promise((resolve, reject) => {
    iframe.addEventListener("load", () => resolve(), { once: true });
    iframe.addEventListener("error", (event) => reject(event.error), {
      once: true,
    });
    if (!iframe.isConnected) {
      document.body.appendChild(iframe);
    }
    iframe.src = url.toString();
  });
}
