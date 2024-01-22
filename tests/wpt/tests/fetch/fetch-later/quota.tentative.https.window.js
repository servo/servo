// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/pending-beacon/resources/pending_beacon-helper.js

'use strict';

const kQuotaPerOrigin = 64 * 1024;  // 64 kilobytes per spec.
const {ORIGIN, HTTPS_NOTSAMESITE_ORIGIN} = get_host_info();

// Runs a test case that cover a single fetchLater() call with `body` in its
// request payload. The call is not expected to throw any errors.
function fetchLaterPostTest(body, description) {
  test(() => {
    const controller = new AbortController();
    const result = fetchLater(
        '/fetch-later',
        {method: 'POST', signal: controller.signal, body: body});
    assert_false(result.activated);
    // Release quota taken by the pending request for subsequent tests.
    controller.abort();
  }, description);
}

// Test small payload for each supported data types.
for (const [dataType, skipCharset] of Object.entries(
         BeaconDataTypeToSkipCharset)) {
  fetchLaterPostTest(
      makeBeaconData(generateSequentialData(0, 1024, skipCharset), dataType),
      `A fetchLater() call accept small data in POST request of ${dataType}.`);
}

// Test various size of payloads for the same origin.
for (const dataType in BeaconDataType) {
  if (dataType !== BeaconDataType.FormData &&
      dataType !== BeaconDataType.URLSearchParams) {
    // Skips FormData & URLSearchParams, as browser adds extra bytes to them
    // in addition to the user-provided content. It is difficult to test a
    // request right at the quota limit.
    fetchLaterPostTest(
        // Generates data that is exactly 64 kilobytes.
        makeBeaconData(generatePayload(kQuotaPerOrigin), dataType),
        `A single fetchLater() call takes up the per-origin quota for its ` +
            `body of ${dataType}.`);
  }
}

// Test empty payload.
for (const dataType in BeaconDataType) {
  test(
      () => {
        assert_throws_js(
            TypeError, () => fetchLater('/', {method: 'POST', body: ''}));
      },
      `A single fetchLater() call does not accept empty data in POST request ` +
          `of ${dataType}.`);
}

// Test oversized payload.
for (const dataType in BeaconDataType) {
  test(
      () => {
        assert_throws_dom(
            'QuotaExceededError',
            () => fetchLater('/fetch-later', {
              method: 'POST',
              // Generates data that exceeds 64 kilobytes.
              body:
                  makeBeaconData(generatePayload(kQuotaPerOrigin + 1), dataType)
            }));
      },
      `A single fetchLater() call is not allowed to exceed per-origin quota ` +
          `for its body of ${dataType}.`);
}

// Test accumulated oversized request.
for (const dataType in BeaconDataType) {
  test(
      () => {
        const controller = new AbortController();
        // Makes the 1st call that sends only half of allowed quota.
        fetchLater('/fetch-later', {
          method: 'POST',
          signal: controller.signal,
          body: makeBeaconData(generatePayload(kQuotaPerOrigin / 2), dataType)
        });

        // Makes the 2nd call that sends half+1 of allowed quota.
        assert_throws_dom('QuotaExceededError', () => {
          fetchLater('/fetch-later', {
            method: 'POST',
            signal: controller.signal,
            body: makeBeaconData(
                generatePayload(kQuotaPerOrigin / 2 + 1), dataType)
          });
        });
        // Release quota taken by the pending requests for subsequent tests.
        controller.abort();
      },
      `The 2nd fetchLater() call is not allowed to exceed per-origin quota ` +
          `for its body of ${dataType}.`);
}

// Test various size of payloads across different origins.
for (const dataType in BeaconDataType) {
  test(
      () => {
        const controller = new AbortController();
        // Makes the 1st call that sends only half of allowed quota.
        fetchLater('/fetch-later', {
          method: 'POST',
          signal: controller.signal,
          body: makeBeaconData(generatePayload(kQuotaPerOrigin / 2), dataType)
        });

        // Makes the 2nd call that sends half+1 of allowed quota, but to a
        // different origin.
        fetchLater(`${HTTPS_NOTSAMESITE_ORIGIN}/fetch-later`, {
          method: 'POST',
          signal: controller.signal,
          body:
              makeBeaconData(generatePayload(kQuotaPerOrigin / 2 + 1), dataType)
        });
        // Release quota taken by the pending requests for subsequent tests.
        controller.abort();
      },
      `The 2nd fetchLater() call to another origin does not exceed per-origin` +
          ` quota for its body of ${dataType}.`);
}
