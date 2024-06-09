// META: title=Permissions Policy "deferred-fetch" is disabled
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

const description = 'Permissions policy header: "deferred-fetch=()"';

parallelPromiseTest(async _ => {
  // Request the browser to fetchLater() immediately, which is not allowed.
  assert_throws_dom(
      'NotAllowedError', () => fetchLater('/', {activateAfter: 0}));
}, `${description} disallows fetchLater() in the top-level document.`);

async_test(t => {
  test_feature_availability(
      'fetchLater()', t, getDeferredFetchPolicyInIframeHelperUrl(HTTPS_ORIGIN),
      expect_feature_unavailable_default);
}, `${description} disallows fetchLater() in the same-origin iframe.`);

async_test(t => {
  test_feature_availability(
      'fetchLater()', t,
      getDeferredFetchPolicyInIframeHelperUrl(HTTPS_NOTSAMESITE_ORIGIN),
      expect_feature_unavailable_default);
}, `${description} disallows fetchLater() in the cross-origin iframe.`);
