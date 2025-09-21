// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/fetch/fetch-later/resources/fetch-later-helper.js
'use strict';

const SMALL_REQUEST_BODY_SIZE = 20;

for (const dataType in BeaconDataType) {
  // Test making a POST request with small payload.
  parallelPromiseTest(
      async _ => expectFetchLater({
        activateAfter: 0,
        method: 'POST',
        body:
            makeBeaconData(generatePayload(SMALL_REQUEST_BODY_SIZE), dataType),
      }),
      `fetchLater() accepts small payload in a POST request body of ${
          dataType}.`);
}
