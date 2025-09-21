// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/fetch/fetch-later/resources/fetch-later-helper.js
'use strict';

const {HTTPS_NOTSAMESITE_ORIGIN} = get_host_info();
const SMALL_REQUEST_BODY_SIZE = 20;

for (const dataType in BeaconDataType) {
  // In a cross-origin iframe, test making a POST request with small
  // payload.
  parallelPromiseTest(
      async _ => await loadFetchLaterIframe(HTTPS_NOTSAMESITE_ORIGIN, {
        activateAfter: 0,
        method: 'POST',
        bodyType: dataType,
        bodySize: SMALL_REQUEST_BODY_SIZE,
      }),
      `fetchLater() accepts payload[size=${
          SMALL_REQUEST_BODY_SIZE}] in a POST request body of ${
          dataType} in a default cross-origin iframe.`);
}
