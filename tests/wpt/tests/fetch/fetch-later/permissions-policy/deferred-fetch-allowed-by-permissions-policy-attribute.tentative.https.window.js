// META: title=Permissions Policy "deferred-fetch" is allowed by allow attribute
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

const description = 'Permissions policy "deferred-fetch"';
const attribute = 'allow="deferred-fetch" attribute';

async_test(
    t => {
      test_feature_availability(
          'fetchLater()', t,
          getDeferredFetchPolicyInIframeHelperUrl(HTTPS_ORIGIN),
          expect_feature_available_default, /*feature_name=*/ 'deferred-fetch');
    },
    `${description} can be enabled in the same-origin iframe using ${
        attribute}.`);

async_test(
    t => {
      test_feature_availability(
          'fetchLater()', t,
          getDeferredFetchPolicyInIframeHelperUrl(HTTPS_NOTSAMESITE_ORIGIN),
          expect_feature_available_default, /*feature_name=*/ 'deferred-fetch');
    },
    `${description} can be enabled in the cross-origin iframe using ${
        attribute}.`);
