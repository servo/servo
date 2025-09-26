// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/fetch/fetch-later/resources/fetch-later-helper.js
// META: script=/fetch/fetch-later/quota/resources/helper.js
'use strict';

const {HTTPS_ORIGIN} = get_host_info();

// Skips FormData & URLSearchParams, as browser adds extra bytes to them
// in addition to the user-provided content. It is difficult to test a
// request right at the quota limit.
// Skips File & Blob as it's difficult to estimate what additional data are
// added into them.
const dataType = BeaconDataType.String;

// Request headers are counted into total request size.
const headers = new Headers({'Content-Type': 'text/plain;charset=UTF-8'});

const requestUrl = `${HTTPS_ORIGIN}/`;
const quota = getRemainingQuota(QUOTA_PER_ORIGIN, requestUrl, headers);
const SMALL_REQUEST_BODY_SIZE = 4 * 1024;  // 4KB.

// This test validates the correct behavior for a sandboxed iframe that includes
// the 'allow-same-origin' token.
//
// Such an iframe should be treated as same-origin. Therefore, it should share
// the parent document's primary 64KB quota pool for fetchLater() requests.
//
// The test works by first having the parent document consume its entire quota.
// Then, it creates the 'allow-same-origin' sandboxed iframe and attempts to
// send a small request.
//
// The expected result is that the iframe's request is REJECTED with a
// QuotaExceededError, proving that it is correctly sharing the parent's
// (already exhausted) quota.
promise_test(async test => {
  const controller = new AbortController();
  test.add_cleanup(() => controller.abort());

  // Step 1: Exhaust the parent frame's entire fetchLater() quota.
  fetchLater(requestUrl, {
    method: 'POST',
    signal: controller.signal,
    body: makeBeaconData(generatePayload(quota), dataType),
    referrer: '',  // Referrer is part of the quota, so we control it.
  });

  // Step 2: From a sandboxed 'allow-same-origin' iframe, attempt to send a
  // small request. This should fail as the shared quota is already gone.
  await loadFetchLaterIframe(
      HTTPS_ORIGIN,  // The iframe's src is same-origin.
      {
        targetUrl: requestUrl,
        activateAfter: 0,
        method: 'POST',
        bodyType: dataType,
        bodySize: SMALL_REQUEST_BODY_SIZE,
        referrer: '',
        sandbox: 'allow-scripts allow-same-origin',
        expect: new FetchLaterIframeExpectation(
            FetchLaterExpectationType.ERROR_DOM, 'QuotaExceededError'),
      });
}, `A sandboxed iframe with 'allow-same-origin' should be treated as same-origin and share the parent's quota.`);
