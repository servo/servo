// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/fetch/fetch-later/resources/fetch-later-helper.js
'use strict';

const {HTTPS_ORIGIN, HTTPS_NOTSAMESITE_ORIGIN} = get_host_info();

// In a cross-origin iframe, test making a POST request with empty
// payload, which is not accepted by fetchLater API.
for (const dataType in BeaconDataType) {
  parallelPromiseTest(
      async _ => await loadFetchLaterIframe(HTTPS_NOTSAMESITE_ORIGIN, {
        activateAfter: 0,
        method: 'POST',
        bodyType: dataType,
        bodySize: 0,
      }),
      `fetchLater() accepts an empty POST request body of ${
          dataType} in a default cross-origin iframe.`);
}
