// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=./resources/pending_beacon-helper.js

'use strict';

const {HTTPS_ORIGIN, HTTPS_NOTSAMESITE_ORIGIN} = get_host_info();

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid, {host: HTTPS_ORIGIN});

  let beacon = new PendingGetBeacon(url);
  beacon.sendNow();

  await expectBeacon(uuid, {count: 1});
}, 'PendingGetBeacon: same-origin');

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(
      uuid, {host: HTTPS_NOTSAMESITE_ORIGIN, expectOrigin: HTTPS_ORIGIN});

  let beacon = new PendingGetBeacon(url);
  beacon.sendNow();

  await expectBeacon(uuid, {count: 1});
}, 'PendingGetBeacon: cross-origin');
