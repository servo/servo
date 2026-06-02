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
const SMALL_REQUEST_BODY_SIZE = 4 * 1024;  // 4KB, well within minimal quota.

// This test validates the correct behavior for a sandboxed iframe without the
// 'allow-same-origin' token.
//
// Such an iframe should be treated as cross-origin, even if its `src` attribute
// points to a same-origin URL. Therefore, it should be allocated its own
// separate "minimal quota" (8KB) for fetchLater() requests and should NOT share
// the parent document's primary quota pool.
//
// The test works by first completely exhausting the parent document's quota.
// Then, it creates the sandboxed iframe and attempts to send a small request
// from it.
//
// The expected result is that the iframe's request SUCCEEDS, because it should
// have its own independent 8KB quota to use.
//
// NOTE: This test will FAIL until the underlying bug is fixed. The bug causes
// the sandboxed iframe to be incorrectly treated as same-origin, making it try
// to use the parent's already-exhausted quota, which leads to a premature
// QuotaExceededError.
promise_test(async test => {
  const controller = new AbortController();
  test.add_cleanup(() => controller.abort());

  // Step 1: Exhaust the parent frame's entire fetchLater() quota.
  fetchLater(requestUrl, {
    method: 'POST',
    signal: controller.signal,
    body: makeBeaconData(generatePayload(quota), dataType),
    referrer: '',  // Referrer is part of the quota, so we control it.
  });

  // Step 2: Create a sandboxed iframe and attempt a small fetchLater()
  // call from it. This should succeed as it should have its own 8KB quota.
  await loadFetchLaterIframe(
      HTTPS_ORIGIN,  // The iframe's src is same-origin.
      {
        targetUrl: requestUrl,
        activateAfter: 0,
        method: 'POST',
        bodyType: dataType,
        bodySize: SMALL_REQUEST_BODY_SIZE,
        referrer: '',
        sandbox: 'allow-scripts',  // Sandboxed, but NOT allow-same-origin.
      });
}, `A sandboxed iframe (without allow-same-origin) should be treated as cross-origin and have its own minimal quota.`);
