// @ts-check
// Import the types from the TypeScript file
/**
 * @typedef {import('../dc-types').ProviderType} ProviderType
 * @typedef {import('../dc-types').IdentityRequestProvider} IdentityRequestProvider
 * @typedef {import('../dc-types').DigitalCredentialRequestOptions} DigitalCredentialRequestOptions
 * @typedef {import('../dc-types').CredentialRequestOptions} CredentialRequestOptions
 * @typedef {import('../dc-types').SendMessageData} SendMessageData
 */
/**
 * @param {ProviderType | ProviderType[]} [providersToUse=["default"]]
 * @param {CredentialMediationRequirement} [mediation="required"]
 * @returns {CredentialRequestOptions}
 */
export function makeGetOptions(providersToUse = ["default"], mediation = "required") {
  if (typeof providersToUse === "string") {
    if (providersToUse === "default" || providersToUse === "openid4vp"){
      return makeGetOptions([providersToUse]);
    }
  }
  if (!Array.isArray(providersToUse) || !providersToUse?.length) {
    return { digital: { providers: providersToUse }, mediation };
  }
  const providers = [];
  for (const provider of providersToUse) {
    switch (provider) {
      case "openid4vp":
        providers.push(makeOID4VPDict());
        break;
      case "default":
        providers.push(makeIdentityRequestProvider(undefined, undefined));
        break;
      default:
        throw new Error(`Unknown provider type: ${provider}`);
    }
  }
  return { digital: { providers }, mediation };
}
/**
 *
 * @param {string} protocol
 * @param {object} request
 * @returns {IdentityRequestProvider}
 */
function makeIdentityRequestProvider(protocol = "protocol", request = {}) {
  return {
    protocol,
    request,
  };
}

/**
 * Representation of a digital identity object with an OpenID4VP provider.
 *
 * @returns {IdentityRequestProvider}
 **/
function makeOID4VPDict() {
  return makeIdentityRequestProvider("openid4vp", {
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
