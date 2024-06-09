'use strict';

/**
 * Returns an URL to a document that can be used to initialize an iframe to test
 * whether the "deferred-fetch"policy is enabled.
 */
function getDeferredFetchPolicyInIframeHelperUrl(iframeOrigin) {
  if (!iframeOrigin.endsWith('/')) {
    iframeOrigin += '/';
  }
  return `${
      iframeOrigin}fetch/fetch-later/permissions-policy/resources/permissions-policy-deferred-fetch.html`;
}
