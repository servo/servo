// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js
// META: script=/pending-beacon/resources/pending_beacon-helper.js

'use strict';

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);
  // Sets no option to test the default behavior when a document enters BFCache.
  const helper = new RemoteContextHelper();
  // Opens a window with noopener so that BFCache will work.
  const rc1 = await helper.addWindow(
      /*config=*/ null, /*options=*/ {features: 'noopener'});

  // Creates a fetchLater request with default config in remote, which should
  // only be sent on page discarded (not on entering BFCache).
  await rc1.executeScript(url => {
    fetchLater(url);
    // Add a pageshow listener to stash the BFCache event.
    window.addEventListener('pageshow', e => {
      window.pageshowEvent = e;
    });
  }, [url]);
  // Navigates away to let page enter BFCache.
  const rc2 = await rc1.navigateToNew();
  // Navigate back.
  await rc2.historyBack();
  // Verify that the page was BFCached.
  assert_true(await rc1.executeScript(() => {
    return window.pageshowEvent.persisted;
  }));

  await expectBeacon(uuid, {count: 0});
}, `fetchLater() does not send on page entering BFCache.`);

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);
  const helper = new RemoteContextHelper();
  // Opens a window with noopener so that BFCache will work.
  const rc1 = await helper.addWindow(
      /*config=*/ null, /*options=*/ {features: 'noopener'});

  // When the remote is BFCached, creates a fetchLater request w/
  // activationTimeout = 0s. It should be sent out immediately.
  await rc1.executeScript(url => {
    window.addEventListener('pagehide', e => {
      if (e.persisted) {
        fetchLater(url, {activationTimeout: 0});
      }
    });
    // Add a pageshow listener to stash the BFCache event.
    window.addEventListener('pageshow', e => {
      window.pageshowEvent = e;
    });
  }, [url]);
  // Navigates away to trigger request sending.
  const rc2 = await rc1.navigateToNew();
  // Navigate back.
  await rc2.historyBack();
  // Verify that the page was BFCached.
  assert_true(await rc1.executeScript(() => {
    return window.pageshowEvent.persisted;
  }));

  await expectBeacon(uuid, {count: 1});
}, `Call fetchLater() when BFCached with activationTimeout=0 sends immediately.`);

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);
  // Sets no option to test the default behavior when a document gets discarded
  // on navigated away.
  const helper = new RemoteContextHelper();
  // Opens a window without BFCache.
  const rc1 = await helper.addWindow();

  // Creates a fetchLater request in remote which should only be sent on
  // navigating away.
  await rc1.executeScript(url => {
    fetchLater(url);
    // Add a pageshow listener to stash the BFCache event.
    window.addEventListener('pageshow', e => {
      window.pageshowEvent = e;
    });
  }, [url]);
  // Navigates away to trigger request sending.
  const rc2 = await rc1.navigateToNew();
  // Navigate back.
  await rc2.historyBack();
  // Verify that the page was NOT BFCached.
  assert_equals(undefined, await rc1.executeScript(() => {
    return window.pageshowEvent;
  }));

  await expectBeacon(uuid, {count: 1});
}, `fetchLater() sends on navigating away a page w/o BFCache.`);
