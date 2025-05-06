// @ts-check
// Import the types from the TypeScript file
/**
 * @typedef {import('../dc-types').GetProtocol} GetProtocol
 * @typedef {import('../dc-types').DigitalCredentialGetRequest} DigitalCredentialGetRequest
 * @typedef {import('../dc-types').DigitalCredentialRequestOptions} DigitalCredentialRequestOptions
 * @typedef {import('../dc-types').CredentialRequestOptions} CredentialRequestOptions
 * @typedef {import('../dc-types').CreateProtocol} CreateProtocol
 * @typedef {import('../dc-types').DigitalCredentialCreateRequest} DigitalCredentialCreateRequest
 * @typedef {import('../dc-types').CredentialCreationOptions} CredentialCreationOptions
 * @typedef {import('../dc-types').DigitalCredentialCreationOptions} DigitalCredentialCreationOptions
 * @typedef {import('../dc-types').SendMessageData} SendMessageData
 */

/**
 * Internal helper to build the request array from validated input.
 * Assumes requestsInputArray is a non-empty array of strings.
 * @private
 * @param {string[]} requestsInputArray - An array of request type strings.
 * @param {string} mediation - The mediation requirement.
 * @param {object} requestMapping - The specific mapping object for the operation type.
 * @returns {{ digital: { requests: any[] }, mediation: string }} - The final options structure.
 * @throws {Error} If an unknown request type string is encountered within the array.
 */
function _makeOptionsInternal(requestsInputArray, mediation, requestMapping) {
  const requests = [];
  for (const request of requestsInputArray) {
    const factoryFunction = requestMapping[request];
    if (factoryFunction) {
      requests.push(factoryFunction()); // Call the mapped function
    } else {
      // This error means a string *within the array* was unknown
      throw new Error(`Unknown request type within array: ${request}`);
    }
  }
  return { digital: { requests }, mediation };
}

const allMappings = {
  get: {
    "openid4vp": () => makeOID4VPDict(),
    "default": () => makeDigitalCredentialGetRequest(undefined, undefined),
  },
  create: {
    "openid4vci": () => makeOID4VCIDict(),
    "default": () => makeDigitalCredentialCreateRequest(),
  },
};

/**
 * Internal unified function to handle option creation logic.
 * Routes calls from specific public functions.
 * @private
 * @param {'get' | 'create'} type - The type of operation.
 * @param {string | string[]} [requestsToUse] - Raw input for request types from public function.
 * @param {string} mediation - Mediation requirement (default handled by public function).
 * @returns {{ digital: { requests: any[] }, mediation: string }}
 * @throws {Error} If type is invalid internally, or input strings are invalid.
 */
function _makeOptionsUnified(type, requestsToUse, mediation) {
  // 1. Get mapping (Type validation primarily happens via caller)
  const mapping = allMappings[type];
   // Added safety check, though public functions should prevent this.
  if (!mapping) {
    throw new Error(`Internal error: Invalid options type specified: ${type}`);
  }

  // 2. Handle default for requestsToUse
  const actualRequestsToUse = requestsToUse === undefined ? ["default"] : requestsToUse;

  // 3. Handle single string input
  if (typeof actualRequestsToUse === 'string') {
    if (mapping[actualRequestsToUse]) {
      // Valid single string: Pass as array to the core array helper
      return _makeOptionsInternal([actualRequestsToUse], mediation, mapping);
    } else {
      // Invalid single string for this type
      throw new Error(`Unknown request type string '${actualRequestsToUse}' provided for operation type '${type}'`);
    }
  }

  // 4. Handle array input
  if (Array.isArray(actualRequestsToUse)) {
    if (actualRequestsToUse.length === 0) {
      // Handle empty array explicitly
      return { digital: { requests: [] }, mediation };
    }
    // Pass valid non-empty array to the core array helper
    return _makeOptionsInternal(actualRequestsToUse, mediation, mapping);
  }

  // 5. Handle invalid input types (neither string nor array)
  return { digital: { requests: [] }, mediation };
}

/**
 * Creates options for getting credentials.
 * @export
 * @param {string | string[]} [requestsToUse] - Request types ('default', 'openid4vp', or an array). Defaults to ['default'].
 * @param {string} [mediation="required"] - Credential mediation requirement ("required", "optional", "silent").
 * @returns {{ digital: { requests: any[] }, mediation: string }}
 */
export function makeGetOptions(requestsToUse, mediation = "required") {
  // Pass type 'get', the user's input, and the final mediation value
  return _makeOptionsUnified('get', requestsToUse, mediation);
}

/**
 * Creates options for creating credentials.
 * @export
 * @param {string | string[]} [requestsToUse] - Request types ('default', 'openid4vci', or an array). Defaults to ['default'].
 * @param {string} [mediation="required"] - Credential mediation requirement ("required", "optional", "silent").
 * @returns {{ digital: { requests: any[] }, mediation: string }} // Adjust inner array type if known
 */
export function makeCreateOptions(requestsToUse, mediation = "required") {
  // Pass type 'create', the user's input, and the final mediation value
  return _makeOptionsUnified('create', requestsToUse, mediation);
}

/**
 *
 * @param {string} protocol
 * @param {object} data
 * @returns {DigitalCredentialGetRequest}
 */
function makeDigitalCredentialGetRequest(protocol = "protocol", data = {}) {
  return {
    protocol,
    data,
  };
}

/**
 * Representation of an OpenID4VP request.
 *
 * @returns {DigitalCredentialGetRequest}
 **/
function makeOID4VPDict() {
  return makeDigitalCredentialGetRequest("openid4vp", {
    // Canonical example of an OpenID4VP request coming soon.
  });
}

/**
 *
 * @param {string} protocol
 * @param {object} data
 * @returns {DigitalCredentialCreateRequest}
 */
function makeDigitalCredentialCreateRequest(protocol = "protocol", data = {}) {
  return {
    protocol,
    data,
  };
}

/**
 * Representation of an OpenID4VCI request.
 *
 * @returns {DigitalCredentialCreateRequest}
 **/
function makeOID4VCIDict() {
  return makeDigitalCredentialCreateRequest("openid4vci", {
    // Canonical example of an OpenID4VCI request coming soon.
  });
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
          "iframe.contentWindow is undefined, cannot send message (something is wrong with the test that called this)."
        )
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
    iframe.addEventListener("load", resolve, { once: true });
    iframe.addEventListener("error", reject, { once: true });
    if (!iframe.isConnected) {
      document.body.appendChild(iframe);
    }
    iframe.src = url.toString();
  });
}
