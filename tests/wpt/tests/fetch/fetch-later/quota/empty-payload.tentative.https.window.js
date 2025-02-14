// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/fetch/fetch-later/resources/fetch-later-helper.js
'use strict';

const {HTTPS_ORIGIN, HTTPS_NOTSAMESITE_ORIGIN} = get_host_info();

// Test making a HTTP POST request with empty payload, which is not accepted by
// fetchLater API.
for (const dataType in BeaconDataType) {
  const requestInit = {
    activateAfter: 0,
    method: 'POST',
    body: makeBeaconData('', dataType)
  };

  if (dataType === BeaconDataType.FormData) {
    // An empty FormData object serializes to non-empty String. Hence, there
    // will be no error thrown from fetchLater.
    parallelPromiseTest(async _ => {
      expectFetchLater(requestInit);
    }, `fetchLater() accepts a non-empty POST request body of ${dataType}.`);
    continue;
  }
  test(
      () => assert_throws_js(TypeError, () => fetchLater('/', requestInit)),
      `fetchLater() does not accept an empty POST request body of ${
          dataType}.`);
}

// Test making HTTP non-POST requests, which has no payload and should be
// accepted by fetchLater API.
for (const method of ['GET', 'DELETE', 'PUT']) {
  parallelPromiseTest(
      async _ => expectFetchLater({activateAfter: 0, method: method}),
      `fetchLater() accept a ${method} request.`);
}
