// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/fetch/fetch-later/resources/fetch-later-helper.js
'use strict';

const {HTTPS_ORIGIN, HTTPS_NOTSAMESITE_ORIGIN} = get_host_info();

// In a same-origin iframe, test making a POST request with empty payload,
// which is not accepted by fetchLater API.
for (const dataType in BeaconDataType) {
  if (dataType === BeaconDataType.FormData) {
    // An empty FormData object serializes to non-empty String. Hence, there
    // will be no error thrown from fetchLater.
    parallelPromiseTest(
        async _ => await loadFetchLaterIframe(HTTPS_ORIGIN, {
          activateAfter: 0,
          method: 'POST',
          bodyType: dataType,
          bodySize: 0,
        }),
        `fetchLater() accepts a non-empty POST request body of ${
            dataType} in same-origin iframe.`);
    continue;
  }

  parallelPromiseTest(
      async _ => await loadFetchLaterIframe(HTTPS_ORIGIN, {
        activateAfter: 0,
        method: 'POST',
        bodyType: dataType,
        bodySize: 0,
        expect: new FetchLaterIframeExpectation(
            FetchLaterExpectationType.ERROR_JS, TypeError),
      }),
      `fetchLater() does not accept empty POST request body of ${
          dataType} in same-origin iframe.`);
}
