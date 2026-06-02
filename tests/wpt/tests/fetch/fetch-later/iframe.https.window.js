// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=/fetch/fetch-later/resources/fetch-later-helper.js

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
