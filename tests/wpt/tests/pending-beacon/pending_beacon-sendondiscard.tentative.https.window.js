// META: script=/common/utils.js
// META: script=./resources/pending_beacon-helper.js

'use strict';

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);
  const numPerMethod = 20;
  const total = numPerMethod * 2;

  // Loads an iframe that creates `numPerMethod` GET & POST beacons.
  const iframe = await loadScriptAsIframe(`
    const url = "${url}";
    for (let i = 0; i < ${numPerMethod}; i++) {
      let get = new PendingGetBeacon(url);
      let post = new PendingPostBeacon(url);
    }
  `);

  // Delete the iframe to trigger beacon sending.
  document.body.removeChild(iframe);

  // The iframe should have sent all beacons.
  await expectBeacon(uuid, {count: total});
}, 'A discarded document sends all its beacons with default config.');

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);

  // Loads an iframe that creates a GET beacon,
  // then sends it out with `sendNow()`.
  const iframe = await loadScriptAsIframe(`
    const url = "${url}";
    let beacon = new PendingGetBeacon(url);
    beacon.sendNow();
  `);

  // Delete the document and verify no more beacons are sent.
  document.body.removeChild(iframe);

  // The iframe should have sent only 1 beacon.
  await expectBeacon(uuid, {count: 1});
}, 'A discarded document does not send an already sent beacon.');

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);
  const numPerMethod = 20;
  const total = numPerMethod * 2;

  // Loads an iframe that creates `numPerMethod` GET & POST beacons with
  // different timeouts.
  const iframe = await loadScriptAsIframe(`
    const url = "${url}";
    for (let i = 0; i < ${numPerMethod}; i++) {
      let get = new PendingGetBeacon(url, {timeout: 100*i});
      let post = new PendingPostBeacon(url, {timeout: 100*i});
    }
  `);

  // Delete the iframe to trigger beacon sending.
  document.body.removeChild(iframe);

  // Even beacons are configured with different timeouts,
  // the iframe should have sent all beacons when it is discarded.
  await expectBeacon(uuid, {count: total});
}, `A discarded document sends all its beacons of which timeouts are not
    default.`);

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);
  const numPerMethod = 20;
  const total = numPerMethod * 2;

  // Loads an iframe that creates `numPerMethod` GET & POST beacons with
  // different backgroundTimeouts.
  const iframe = await loadScriptAsIframe(`
    const url = "${url}";
    for (let i = 0; i < ${numPerMethod}; i++) {
      let get = new PendingGetBeacon(url, {backgroundTimeout: 100*i});
      let post = new PendingPostBeacon(url, {backgroundTimeout: 100*i});
    }
  `);

  // Delete the iframe to trigger beacon sending.
  document.body.removeChild(iframe);

  // Even beacons are configured with different backgroundTimeouts,
  // the iframe should have sent all beacons when it is discarded.
  await expectBeacon(uuid, {count: total});
}, `A discarded document sends all its beacons of which backgroundTimeouts are
    not default.`);
