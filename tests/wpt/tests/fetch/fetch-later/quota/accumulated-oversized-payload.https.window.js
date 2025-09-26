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
const halfQuota = Math.ceil(quota / 2);


// Tests that a reporting origin only allow queuing requests within its quota.
test(
    () => {
      const controller = new AbortController();

      // Queues with the 1st call (POST) that sends max/2 quota.
      fetchLater(requestUrl, {
        method: 'POST',
        signal: controller.signal,
        body: makeBeaconData(generatePayload(halfQuota), dataType),
        // Required, as the size of referrer also take up quota.
        referrer: '',
      });

      // Makes the 2nd call (POST) to the same reporting origin that sends
      // max bytes, which should be rejected.
      assert_throws_quotaexceedederror(() => {
        fetchLater(requestUrl, {
          method: 'POST',
          signal: controller.signal,
          body: makeBeaconData(generatePayload(quota), dataType),
          // Required, as the size of referrer also take up quota.
          referrer: '',
        });
      }, null, null);

      // Makes the 3rd call (GET) to the same reporting origin, where its
      // request size is len(requestUrl) + headers, which should be accepted.
      fetchLater(requestUrl, {
        method: 'GET',
        signal: controller.signal,
        // Required, as the size of referrer also take up quota.
        referrer: '',
      });

      // Release quota taken by the pending requests for subsequent tests.
      controller.abort();
    },
    `The 2nd fetchLater(same-origin) call in the top-level document is not allowed to exceed per-origin quota for its POST body of ${
        dataType}.`);
