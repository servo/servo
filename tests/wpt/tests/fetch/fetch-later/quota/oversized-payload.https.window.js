// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/fetch/fetch-later/resources/fetch-later-helper.js
// META: script=/fetch/fetch-later/quota/resources/helper.js
'use strict';

const OVERSIZED_REQUEST_BODY_SIZE = QUOTA_PER_ORIGIN + 1;

for (const dataType in BeaconDataType) {
  // Test making a POST request with oversized payload, which should be rejected
  // by fetchLater API.
  test(() => {
    assert_throws_quotaexceedederror(() => {
      fetchLater('/', {
        activateAfter: 0,
        method: 'POST',
        body: makeBeaconData(
            generatePayload(OVERSIZED_REQUEST_BODY_SIZE), dataType),
      });
    }, null, null);
  }, `fetchLater() does not accept payload[size=${
          OVERSIZED_REQUEST_BODY_SIZE}] exceeding per-origin quota in a POST request body of ${
          dataType}.`);
}
