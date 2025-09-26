// META: title=Permissions Policy "deferred-fetch" is allowed to redirect by allow attribute
// META: script=/permissions-policy/resources/permissions-policy.js
// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=/fetch/fetch-later/resources/fetch-later-helper.js
// META: script=/fetch/fetch-later/permissions-policy/resources/helper.js
// META: timeout=long
'use strict';

const {
  HTTPS_ORIGIN,
  HTTPS_NOTSAMESITE_ORIGIN,
} = get_host_info();

const baseUrl = '/permissions-policy/resources/redirect-on-load.html#';
// https://whatpr.org/fetch/1647.html#dom-permissionspolicy-deferred-fetch-minimal
const description = 'Permissions policy allow="deferred-fetch"';

async_test(t => {
  test_feature_availability(
      'fetchLater()', t,
      getDeferredFetchPolicyInIframeHelperUrl(`${baseUrl}${HTTPS_ORIGIN}`),
      expect_feature_available_default, /*feature_name=*/ 'deferred-fetch');
}, `${description} allows fetchLater() from a redirected same-origin iframe.`);

// By default, "deferred-fetch" is off for cross-origin iframe.
// It requires a Permissions-Policy header in addition to the allow attribute.
async_test(t => {
  test_feature_availability(
      'fetchLater()', t,
      getDeferredFetchPolicyInIframeHelperUrl(
          `${baseUrl}${HTTPS_NOTSAMESITE_ORIGIN}`),
      expect_feature_unavailable_default, /*feature_name=*/ 'deferred-fetch');
}, `${description} disallows fetchLater() from a redirected cross-origin iframe.`);
