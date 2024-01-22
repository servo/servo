// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js
// META: script=./resources/pending_beacon-helper.js

'use strict';

for (const beaconType of BeaconTypes) {
  const beaconName = beaconType.name;

  parallelPromiseTest(async t => {
    const uuid = token();
    const url = generateSetBeaconURL(uuid);
    // backgroundTimeout = 0s means `beacon should be sent out right on
    // entering `hidden` state after navigating away.
    const options = {backgroundTimeout: 0};
    const helper = new RemoteContextHelper();
    // Opens a window with noopener so that BFCache will work.
    const rc1 = await helper.addWindow(
        /*config=*/ null, /*options=*/ {features: 'noopener'});

    // Creates a PendingBeacon in remote which should only be sent on navigating
    // away.
    await rc1.executeScript((beaconName, url, options) => {
      const beacon = beaconName == 'PendingGetBeacon' ?
          new PendingGetBeacon(url, options) :
          new PendingPostBeacon(url, options);
    }, [beaconName, url, options]);

    await expectBeacon(uuid, {count: 0});
  }, `${beaconName}: does not send without page navigation.`);

  parallelPromiseTest(async t => {
    const uuid = token();
    const url = generateSetBeaconURL(uuid);
    // backgroundTimeout = 0s means `beacon should be sent out right on
    // entering `hidden` state after navigating away.
    const options = {backgroundTimeout: 0};
    const helper = new RemoteContextHelper();
    // Opens a window with noopener so that BFCache will work.
    const rc1 = await helper.addWindow(
        /*config=*/ null, /*options=*/ {features: 'noopener'});

    // Creates a PendingBeacon in remote which should only be sent on navigating
    // away.
    await rc1.executeScript((beaconName, url, options) => {
      const beacon = beaconName == 'PendingGetBeacon' ?
          new PendingGetBeacon(url, options) :
          new PendingPostBeacon(url, options);
    }, [beaconName, url, options]);
    // Navigates away to trigger beacon sending.
    rc1.navigateToNew();

    await expectBeacon(uuid, {count: 1});
  }, `${beaconName}: sends on page entering hidden state (w/ BFCache).`);

  parallelPromiseTest(async t => {
    const uuid = token();
    const url = generateSetBeaconURL(uuid);
    // backgroundTimeout = 0s means `beacon should be sent out right on
    // entering `hidden` state after navigating away.
    const options = {backgroundTimeout: 0};
    const helper = new RemoteContextHelper();
    // Opens a window without BFCache.
    const rc1 = await helper.addWindow();

    // Creates a PendingBeacon in remote which should only be sent on navigating
    // away.
    await rc1.executeScript((beaconName, url, options) => {
      const beacon = beaconName == 'PendingGetBeacon' ?
          new PendingGetBeacon(url, options) :
          new PendingPostBeacon(url, options);
    }, [beaconName, url, options]);
    // Navigates away to trigger beacon sending.
    rc1.navigateToNew();

    await expectBeacon(uuid, {count: 1});
  }, `${beaconName}: sends on page entering hidden state (w/o BFCache).`);
}
