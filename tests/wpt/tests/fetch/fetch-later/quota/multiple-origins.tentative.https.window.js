// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/fetch/fetch-later/resources/fetch-later-helper.js
// META: script=/fetch/fetch-later/quota/resources/helper.js
'use strict';

const {HTTPS_ORIGIN, HTTPS_NOTSAMESITE_ORIGIN} = get_host_info();

for (const dataType in BeaconDataType) {
  // Tests multiple request reporting origins within same document.
  test(
      () => {
        const controller = new AbortController();

        // Makes the 1st call (POST) that sends max/2+1 quota to `HTTPS_ORIGIN`.
        fetchLater(`${HTTPS_ORIGIN}/`, {
          method: 'POST',
          signal: controller.signal,
          body: makeBeaconData(
              generatePayload(QUOTA_PER_REPORTING_ORIGIN / 2 + 1), dataType)
        });

        // Makes the 2nd call (POST) that sends max/2+1 of allowed quota, but to
        // a different reporting origin `HTTPS_NOTSAMESITE_ORIGIN`.
        fetchLater(`${HTTPS_NOTSAMESITE_ORIGIN}/`, {
          method: 'POST',
          signal: controller.signal,
          body: makeBeaconData(
              generatePayload(QUOTA_PER_REPORTING_ORIGIN / 2 + 1), dataType)
        });

        // Makes the 3rd call (GET) to `HTTPS_NOTSAMESITE_ORIGIN`.
        fetchLater(`${HTTPS_NOTSAMESITE_ORIGIN}/`, {
          method: 'GET',
          signal: controller.signal,
        });

        // Release quota taken by the pending requests for subsequent tests.
        controller.abort();
      },
      `fetchLater() has per-request-origin quota for its POST body of ${
          dataType}.`);
}
