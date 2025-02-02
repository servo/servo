// @ts-check
// Import the types from the TypeScript file
/**
 * @typedef {import('../dc-types').Protocol} protocol
 * @typedef {import('../dc-types').DigitalCredentialRequest} DigitalCredentialRequest
 * @typedef {import('../dc-types').DigitalCredentialRequestOptions} DigitalCredentialRequestOptions
 * @typedef {import('../dc-types').CredentialRequestOptions} CredentialRequestOptions
 * @typedef {import('../dc-types').SendMessageData} SendMessageData
 */

/**
 * @param {protocol | protocol[]} [requestsToUse=["default"]]
 * @param {CredentialMediationRequirement} [mediation="required"]
 * @returns {CredentialRequestOptions}
 */
export function makeGetOptions(requestsToUse, mediation = "required") {
  if (typeof requestsToUse === "string") {
    if (requestsToUse === "default" || requestsToUse === "openid4vp") {
      return makeGetOptions([requestsToUse], mediation);
    }
  }
  if (!Array.isArray(requestsToUse) || !requestsToUse?.length) {
    return { digital: { requests: requestsToUse }, mediation };
  }
  const requests = [];
  for (const request of requestsToUse) {
    switch (request) {
      case "openid4vp":
        requests.push(makeOID4VPDict());
        break;
      case "default":
        requests.push(makeDigitalCredentialRequest(undefined, undefined));
        break;
      default:
        throw new Error(`Unknown request type: ${request}`);
    }
  }
  return { digital: { requests }, mediation };
}
/**
 *
 * @param {string} protocol
 * @param {object} data
 * @returns {DigitalCredentialRequest}
 */
function makeDigitalCredentialRequest(protocol = "protocol", data = {}) {
  return {
    protocol,
    data,
  };
}

/**
 * Representation of an OpenID4VP request.
 *
 * @returns {DigitalCredentialRequest}
 **/
function makeOID4VPDict() {
  return makeDigitalCredentialRequest("openid4vp", {
    // Canonical example of an OpenID4VP request coming soon.
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
