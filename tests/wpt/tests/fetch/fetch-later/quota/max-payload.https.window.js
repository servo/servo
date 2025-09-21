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

// Test making a POST request with max possible payload.
promise_test(async _ => {
  const uuid = token();
  const requestUrl = generateSetBeaconURL(uuid, {host: HTTPS_ORIGIN});
  await expectFetchLater(
      {
        activateAfter: 0,
        method: 'POST',
        bodySize: getRemainingQuota(QUOTA_PER_ORIGIN, requestUrl, headers),
      },
      {
        targetUrl: requestUrl,
        uuid: uuid,
      });
}, `fetchLater() accepts max payload in a POST request body of ${dataType}.`);

// Test making a POST request with max+1 possible payload.
test(_ => {
  const uuid = token();
  const requestUrl = generateSetBeaconURL(uuid, {host: HTTPS_ORIGIN});

  assert_throws_quotaexceedederror(() => {
    fetchLater(requestUrl, {
          activateAfter: 0,
          method: 'POST',
          body: generatePayload(
              getRemainingQuota(QUOTA_PER_ORIGIN, requestUrl, headers) + 1,
              dataType),
        });
  }, null, null);
}, `fetchLater() rejects max+1 payload in a POST request body of ${dataType}.`);
