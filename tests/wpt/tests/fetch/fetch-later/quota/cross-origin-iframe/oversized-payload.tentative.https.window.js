// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/fetch/fetch-later/resources/fetch-later-helper.js
// META: script=/fetch/fetch-later/quota/resources/helper.js
'use strict';

const {HTTPS_NOTSAMESITE_ORIGIN} = get_host_info();
const OVERSIZED_REQUEST_BODY_SIZE = QUOTA_PER_CROSS_ORIGIN + 1;

for (const dataType in BeaconDataType) {
  // In a cross-origin iframe, test making a POST request with
  // oversized payload, which should be rejected by fetchLater API.
  parallelPromiseTest(
      async _ => await loadFetchLaterIframe(HTTPS_NOTSAMESITE_ORIGIN, {
        activateAfter: 0,
        method: 'POST',
        bodyType: dataType,
        bodySize: OVERSIZED_REQUEST_BODY_SIZE,
        expect: new FetchLaterIframeExpectation(
            FetchLaterExpectationType.ERROR_DOM, 'QuotaExceededError'),
      }),
      `fetchLater() does not accept payload[size=${
          OVERSIZED_REQUEST_BODY_SIZE}] exceeding per-origin quota in a POST request body of ${
          dataType} in a default cross-origin iframe.`);
}
