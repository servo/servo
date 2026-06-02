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


// In a cross-origin iframe, test making a POST request (same origin as parent
// frame) with max possible payload.
promise_test(
    async _ => {
      const uuid = token();
      const requestUrl = generateSetBeaconURL(uuid, {host: HTTPS_ORIGIN});

      await loadFetchLaterIframe(HTTPS_NOTSAMESITE_ORIGIN, {
        targetUrl: requestUrl,
        uuid: uuid,
        activateAfter: 0,
        method: 'POST',
        bodyType: dataType,
        bodySize:
            getRemainingQuota(QUOTA_PER_CROSS_ORIGIN, requestUrl, headers),
        // Required, as the size of referrer also take up quota.
        referrer: '',
      });
    },
    `fetchLater() accepts max payload in a parent-frame-origin POST request body of ${
        dataType} in a default cross-origin iframe.`);

// In a cross-origin iframe, test making a POST request (same origin as parent
// frame) with max possible payload + 1 byte.
promise_test(
    async _ => {
      const uuid = token();
      const requestUrl = generateSetBeaconURL(uuid, {host: HTTPS_ORIGIN});

      await loadFetchLaterIframe(HTTPS_NOTSAMESITE_ORIGIN, {
        targetUrl: requestUrl,
        uuid: uuid,
        activateAfter: 0,
        method: 'POST',
        bodyType: dataType,
        bodySize:
            getRemainingQuota(QUOTA_PER_CROSS_ORIGIN, requestUrl, headers) + 1,
        // Required, as the size of referrer also take up quota.
        referrer: '',
        expect: new FetchLaterIframeExpectation(
            FetchLaterExpectationType.ERROR_DOM, 'QuotaExceededError'),
      });
    },
    `fetchLater() rejects max+1 payload in a parent-frame-origin POST request body of ${
        dataType} in a default cross-origin iframe.`);

// In a cross-origin iframe, test making a POST request (same origin as iframe)
// with max possible payload.
promise_test(
    async _ => {
      const uuid = token();
      const crossOriginRequestUrl =
          generateSetBeaconURL(uuid, {host: HTTPS_NOTSAMESITE_ORIGIN});

      await loadFetchLaterIframe(HTTPS_NOTSAMESITE_ORIGIN, {
        targetUrl: crossOriginRequestUrl,
        uuid: uuid,
        activateAfter: 0,
        method: 'POST',
        bodyType: dataType,
        bodySize: getRemainingQuota(
            QUOTA_PER_CROSS_ORIGIN, crossOriginRequestUrl, headers),
        // Required, as the size of referrer also take up quota.
        referrer: '',
      });
    },
    `fetchLater() accepts max payload in a self-frame-origin POST request body of ${
        dataType} in a default cross-origin iframe.`);

// In a cross-origin iframe, test making a POST request (same origin as iframe)
// with max possible payload + 1 byte.
promise_test(
    async _ => {
      const uuid = token();
      const crossOriginRequestUrl =
          generateSetBeaconURL(uuid, {host: HTTPS_NOTSAMESITE_ORIGIN});

      await loadFetchLaterIframe(HTTPS_NOTSAMESITE_ORIGIN, {
        targetUrl: crossOriginRequestUrl,
        uuid: uuid,
        activateAfter: 0,
        method: 'POST',
        bodyType: dataType,
        bodySize: getRemainingQuota(
                      QUOTA_PER_CROSS_ORIGIN, crossOriginRequestUrl, headers) +
            1,
        // Required, as the size of referrer also take up quota.
        referrer: '',
        expect: new FetchLaterIframeExpectation(
            FetchLaterExpectationType.ERROR_DOM, 'QuotaExceededError'),
      });
    },
    `fetchLater() rejects max+1 payload in a self-frame-origin POST request body of ${
        dataType} in a default cross-origin iframe.`);
