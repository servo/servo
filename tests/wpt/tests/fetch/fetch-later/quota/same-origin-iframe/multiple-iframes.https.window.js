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

// Test 2 direct same-origin iframes use the same quota with top-level document.
promise_test(async _ => {
  const uuid = token();
  const requestUrl = generateSetBeaconURL(uuid, {host: HTTPS_ORIGIN});

  // Queues a max bytes request in the 1st same-origin iframe.
  await loadFetchLaterIframe(HTTPS_ORIGIN, {
    targetUrl: requestUrl,
    method: 'POST',
    bodyType: dataType,
    bodySize: getRemainingQuota(QUOTA_PER_ORIGIN, requestUrl, headers),
    // Required, as the size of referrer also take up quota.
    referrer: '',
  });

  // Queues a max bytes request to `HTTPS_ORIGIN` in the 2nd same-origin iframe,
  // which should be rejected because the target URL's origin `HTTPS_ORIGIN`
  // already has 64kb pending from 1st request.
  await loadFetchLaterIframe(HTTPS_ORIGIN, {
    targetUrl: requestUrl,
    method: 'POST',
    bodyType: dataType,
    bodySize: getRemainingQuota(QUOTA_PER_ORIGIN, requestUrl, headers),
    // Required, as the size of referrer also take up quota.
    referrer: '',
    expect: new FetchLaterIframeExpectation(
        FetchLaterExpectationType.ERROR_DOM, 'QuotaExceededError'),
  });

  // Queues a max bytes request in the root document, which should also be
  // rejected, because the target URL's origin `HTTPS_ORIGIN` already has 64kb
  // pending from 1st request.
  assert_throws_quotaexceedederror(
      () => fetchLater(requestUrl, {
        method: 'POST',
        body: generatePayload(
            getRemainingQuota(QUOTA_PER_ORIGIN, requestUrl, headers), dataType),
        // Required, as the size of referrer also take up quota.
        referrer: '',
      }),
      null, null);

  // Release quota taken by the pending requests for subsequent tests.
  for (const element of document.querySelectorAll('iframe')) {
    element.remove();
  }
}, `fetchLater() request quota are shared by same-origin iframes and root.`);
