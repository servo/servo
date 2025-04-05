'use strict';

/**
 * Returns a URL to a document that can be used to initialize an iframe to test
 * whether the Permissions Policy "deferred-fetch" or "deferred-fetch-minimal"
 * is enabled.
 */
function getDeferredFetchPolicyInIframeHelperUrl(iframeOrigin) {
  if (!iframeOrigin.endsWith('/')) {
    iframeOrigin += '/';
  }
  return `${
      iframeOrigin}fetch/fetch-later/permissions-policy/resources/permissions-policy-deferred-fetch.html`;
}
