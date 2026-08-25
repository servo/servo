// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/fetch/fetch-later/resources/fetch-later-helper.js
// META: script=/fetch/fetch-later/quota/resources/helper.js
'use strict';

const {HTTPS_ORIGIN, HTTPS_NOTSAMESITE_ORIGIN} = get_host_info();

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
const SMALL_REQUEST_BODY_SIZE = 20;

// Tests that a reporting origin's quota is not shared with another cross-origin
// iframe.
promise_test(
    async _ => {
      const controller = new AbortController();

      // Queues with the 1st call (POST) that sends max quota.
      fetchLater(requestUrl, {
        method: 'POST',
        signal: controller.signal,
        body: makeBeaconData(generatePayload(quota), dataType),
        // Required, as the size of referrer also take up quota.
        referrer: '',
      });

      // In a default cross-origin iframe, makes the 2nd call (POST) to the
      // same reporting origin that sends some bytes, which should also be
      // accepted, as the iframe has its own QUOTA_PER_CROSS_ORIGIN for
      // fetchLater.
      await loadFetchLaterIframe(HTTPS_NOTSAMESITE_ORIGIN, {
        targetUrl: requestUrl,
        activateAfter: 0,
        method: 'POST',
        bodyType: dataType,
        bodySize: SMALL_REQUEST_BODY_SIZE,
        // Required, as the size of referrer also take up quota.
        referrer: '',
      });

      // Release quota taken by the pending requests for subsequent tests.
      controller.abort();
    },
    `The 2nd fetchLater(same-origin) call in a default cross-origin child iframe has its owned per-origin quota for a request POST body of ${
        dataType}.`);
