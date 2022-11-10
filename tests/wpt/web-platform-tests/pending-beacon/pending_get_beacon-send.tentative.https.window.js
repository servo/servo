// META: script=/common/utils.js
// META: script=./resources/pending_beacon-helper.js

'use strict';

const baseUrl = `${location.protocol}//${location.host}`;

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);

  let beacon = new PendingGetBeacon('/');

  beacon.setURL(url);
  assert_equals(beacon.url, url);
  beacon.sendNow();

  await expectBeacon(uuid, {count: 1});
}, 'PendingGetBeacon is sent to the updated URL');

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);

  let beacon = new PendingGetBeacon('/0');

  for (let i = 0; i < 10; i++) {
    const transientUrl = `/${i}`;
    beacon.setURL(transientUrl);
    assert_equals(beacon.url, transientUrl);
  }
  beacon.setURL(url);
  assert_equals(beacon.url, url);

  beacon.sendNow();

  await expectBeacon(uuid, {count: 1});
}, 'PendingGetBeacon is sent to the last updated URL');
