// META: title=Permissions Policy "deferred-fetch" default behavior
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

const description = 'Default "deferred-fetch" permissions policy ["self"]';

parallelPromiseTest(async _ => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);

  // Request the browser to fetchLater() immediately.
  fetchLater(url, {activateAfter: 0});

  await expectBeacon(uuid, {count: 1});
}, `${description} allows fetchLater() in the top-level document.`);

async_test(t => {
  test_feature_availability(
      'fetchLater()', t, getDeferredFetchPolicyInIframeHelperUrl(HTTPS_ORIGIN),
      expect_feature_available_default);
}, `${description} allows fetchLater() in the same-origin iframe.`);

async_test(t => {
  test_feature_availability(
      'fetchLater()', t,
      getDeferredFetchPolicyInIframeHelperUrl(HTTPS_NOTSAMESITE_ORIGIN),
      expect_feature_available_default);
}, `${description} allows fetchLater() in the cross-origin iframe.`);
