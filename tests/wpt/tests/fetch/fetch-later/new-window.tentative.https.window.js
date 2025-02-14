// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=/fetch/fetch-later/resources/fetch-later-helper.js

'use strict';

const {
  HTTPS_ORIGIN,
  HTTPS_NOTSAMESITE_ORIGIN,
} = get_host_info();

function fetchLaterPopupUrl(host, targetUrl) {
  return `${host}/fetch/fetch-later/resources/fetch-later.html?url=${
      encodeURIComponent(targetUrl)}&activateAfter=0`;
}

async function receiveMessageFromPopup(url) {
  const expect =
      new FetchLaterIframeExpectation(FetchLaterExpectationType.DONE);
  const messageType = await new Promise((resolve, reject) => {
    window.addEventListener('message', function handler(e) {
      try {
        if (expect.run(e, url)) {
          window.removeEventListener('message', handler);
          resolve(e.data.type);
        }
      } catch (err) {
        reject(err);
      }
    });
  });

  assert_equals(messageType, FetchLaterIframeMessageType.DONE);
}

for (const target of ['', '_blank']) {
  // NOTE: noopener popup window cannot communicate back. It will be too
  // unreliable to only use `expectBeacon()` to test such window.
  for (const features of ['', 'popup']) {
    parallelPromiseTest(
        async t => {
          const uuid = token();
          const url =
              generateSetBeaconURL(uuid, {host: HTTPS_NOTSAMESITE_ORIGIN});

          // Opens a blank popup window that fires a fetchLater request.
          const w = window.open(
              `javascript: fetchLater("${url}", {activateAfter: 0})`, target,
              features);
          await new Promise(resolve => w.addEventListener('load', resolve));

          // The popup should have sent the request.
          await expectBeacon(uuid, {count: 1});
          w.close();
        },
        `A blank window[target='${target}'][features='${
            features}'] can trigger fetchLater.`);

    parallelPromiseTest(
        async t => {
          const uuid = token();
          const popupUrl =
              fetchLaterPopupUrl(HTTPS_ORIGIN, generateSetBeaconURL(uuid));

          // Opens a same-origin popup that fires a fetchLater request.
          const w = window.open(popupUrl, target, features);
          await receiveMessageFromPopup(popupUrl);

          // The popup should have sent the request.
          await expectBeacon(uuid, {count: 1});
          w.close();
        },
        `A same-origin window[target='${target}'][features='${
            features}'] can trigger fetchLater.`);

    parallelPromiseTest(
        async t => {
          const uuid = token();
          const popupUrl = fetchLaterPopupUrl(
              HTTPS_NOTSAMESITE_ORIGIN, generateSetBeaconURL(uuid));

          // Opens a cross-origin popup that fires a fetchLater request.
          const w = window.open(popupUrl, target, features);
          await receiveMessageFromPopup(popupUrl);

          // The popup should have sent the request.
          await expectBeacon(uuid, {count: 1});
          w.close();
        },
        `A cross-origin window[target='${target}'][features='${
            features}'] can trigger fetchLater.`);
  }
}
