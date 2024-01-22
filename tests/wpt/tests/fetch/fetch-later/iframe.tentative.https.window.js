// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=/pending-beacon/resources/pending_beacon-helper.js

'use strict';

const {
  HTTPS_ORIGIN,
  HTTPS_NOTSAMESITE_ORIGIN,
} = get_host_info();

async function loadElement(el) {
  const loaded = new Promise(resolve => el.onload = resolve);
  document.body.appendChild(el);
  await loaded;
}

// `host` may be cross-origin
async function loadFetchLaterIframe(host, targetUrl) {
  const url = `${host}/fetch/fetch-later/resources/fetch-later.html?url=${
      encodeURIComponent(targetUrl)}`;
  const iframe = document.createElement('iframe');
  iframe.src = url;
  await loadElement(iframe);
  return iframe;
}

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);

  // Loads a blank iframe that fires a fetchLater request.
  const iframe = document.createElement('iframe');
  iframe.addEventListener('load', () => {
    fetchLater(url, {activateAfter: 0});
  });
  await loadElement(iframe);

  // The iframe should have sent the request.
  await expectBeacon(uuid, {count: 1});
}, 'A blank iframe can trigger fetchLater.');

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);

  // Loads a same-origin iframe that fires a fetchLater request.
  await loadFetchLaterIframe(HTTPS_ORIGIN, url);

  // The iframe should have sent the request.
  await expectBeacon(uuid, {count: 1});
}, 'A same-origin iframe can trigger fetchLater.');

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);

  // Loads a same-origin iframe that fires a fetchLater request.
  await loadFetchLaterIframe(HTTPS_NOTSAMESITE_ORIGIN, url);

  // The iframe should have sent the request.
  await expectBeacon(uuid, {count: 1});
}, 'A cross-origin iframe can trigger fetchLater.');
