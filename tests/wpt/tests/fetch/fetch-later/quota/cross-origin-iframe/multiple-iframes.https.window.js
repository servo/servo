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

// Test 2 direct cross-origin iframes does not share the same quota, even if
// they are same origin.
promise_test(async _ => {
  const controller = new AbortController();
  const uuid = token();
  const requestUrl = generateSetBeaconURL(uuid, {host: HTTPS_ORIGIN});

  // Queues a max bytes request in the 1st cross-origin iframe.
  await loadFetchLaterIframe(HTTPS_NOTSAMESITE_ORIGIN, {
    targetUrl: requestUrl,
    method: 'POST',
    bodyType: dataType,
    bodySize: getRemainingQuota(QUOTA_PER_CROSS_ORIGIN, requestUrl, headers),
    // Required, as the size of referrer also take up quota.
    referrer: '',
  });

  // Queues a max bytes request in the 2nd cross-origin iframe, which should
  // still be accepted.
  await loadFetchLaterIframe(HTTPS_NOTSAMESITE_ORIGIN, {
    targetUrl: requestUrl,
    method: 'POST',
    bodyType: dataType,
    bodySize: getRemainingQuota(QUOTA_PER_CROSS_ORIGIN, requestUrl, headers),
    // Required, as the size of referrer also take up quota.
    referrer: '',
  });

  // Queues a max bytes request in the root document, which should be
  // accepted.
  fetchLater(requestUrl, {
    method: 'POST',
    body: generatePayload(QUOTA_PER_ORIGIN, dataType),
    signal: controller.signal,
    body: generatePayload(
        getRemainingQuota(QUOTA_PER_ORIGIN, requestUrl, headers), dataType),
    // Required, as the size of referrer also take up quota.
    referrer: '',
  });

  // Release quota taken by the pending requests for subsequent tests.
  document.body.innerHTML = '';
  controller.abort();
}, `fetchLater() request quota are delegated to cross-origin iframes and not shared, even if they are same origin.`);
