// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js
// META: script=/pending-beacon/resources/pending_beacon-helper.js

'use strict';

// NOTE: Due to the restriction of WPT runner, the following tests are all run
// with BackgroundSync off, which is different from some browsers,
// e.g. Chrome, default behavior, as the testing infra does not support enabling
// it.

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
  // Navigates back.
  await rc2.historyBack();
  // Verifies the page was BFCached.
  assert_true(await rc1.executeScript(() => {
    return window.pageshowEvent.persisted;
  }));

  // Theoretically, the request should still be pending thus 0 request received.
  // However, 1 request is sent, as by default the WPT test runner, e.g.
  // content_shell in Chromium, does not enable BackgroundSync permission,
  // resulting in forcing request sending on every navigation.
  await expectBeacon(uuid, {count: 1});
}, `fetchLater() sends on page entering BFCache if BackgroundSync is off.`);

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);
  const helper = new RemoteContextHelper();
  // Opens a window with noopener so that BFCache will work.
  const rc1 = await helper.addWindow(
      /*config=*/ null, /*options=*/ {features: 'noopener'});

  // When the remote is put into BFCached, creates a fetchLater request w/
  // activateAfter = 0s. It should be sent out immediately.
  await rc1.executeScript(url => {
    window.addEventListener('pagehide', e => {
      if (e.persisted) {
        fetchLater(url, {activateAfter: 0});
      }
    });
    // Add a pageshow listener to stash the BFCache event.
    window.addEventListener('pageshow', e => {
      window.pageshowEvent = e;
    });
  }, [url]);
  // Navigates away to trigger request sending.
  const rc2 = await rc1.navigateToNew();
  // Navigates back.
  await rc2.historyBack();
  // Verifies the page was BFCached.
  assert_true(await rc1.executeScript(() => {
    return window.pageshowEvent.persisted;
  }));

  // NOTE: In this case, it does not matter if BackgroundSync is on or off.
  await expectBeacon(uuid, {count: 1});
}, `Call fetchLater() when BFCached with activateAfter=0 sends immediately.`);

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
  // Navigates back.
  await rc2.historyBack();
  // Verifies the page was NOT BFCached.
  assert_equals(undefined, await rc1.executeScript(() => {
    return window.pageshowEvent;
  }));

  // NOTE: In this case, it does not matter if BackgroundSync is on or off.
  await expectBeacon(uuid, {count: 1});
}, `fetchLater() sends on navigating away a page w/o BFCache.`);

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);
  // Sets no option to test the default behavior when a document gets discarded
  // on navigated away.
  const helper = new RemoteContextHelper();
  // Opens a window without BFCache.
  const rc1 = await helper.addWindow();

  // Creates 2 fetchLater requests in remote, and one of them is aborted
  // immediately. The other one should only be sent right on navigating away.
  await rc1.executeScript(url => {
    const controller = new AbortController();
    fetchLater(url, {signal: controller.signal});
    fetchLater(url);
    controller.abort();
    // Add a pageshow listener to stash the BFCache event.
    window.addEventListener('pageshow', e => {
      window.pageshowEvent = e;
    });
  }, [url]);
  // Navigates away to trigger request sending.
  const rc2 = await rc1.navigateToNew();
  // Navigates back.
  await rc2.historyBack();
  // Verifies the page was NOT BFCached.
  assert_equals(undefined, await rc1.executeScript(() => {
    return window.pageshowEvent;
  }));

  // NOTE: In this case, it does not matter if BackgroundSync is on or off.
  await expectBeacon(uuid, {count: 1});
}, `fetchLater() does not send aborted request on navigating away a page w/o BFCache.`);

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);
  const options = {activateAfter: 60000};
  const helper = new RemoteContextHelper();
  // Opens a window with noopener so that BFCache will work.
  const rc1 = await helper.addWindow(
      /*config=*/ null, /*options=*/ {features: 'noopener'});

  // Creates a fetchLater request in remote which should only be sent on
  // navigating away.
  await rc1.executeScript((url) => {
    // Sets activateAfter = 1m to indicate the request should NOT be sent out
    // immediately.
    fetchLater(url, {activateAfter: 60000});
    // Adds a pageshow listener to stash the BFCache event.
    window.addEventListener('pageshow', e => {
      window.pageshowEvent = e;
    });
  }, [url]);
  // Navigates away to trigger request sending.
  const rc2 = await rc1.navigateToNew();
  // Navigates back.
  await rc2.historyBack();
  // Verifies the page was BFCached.
  assert_true(await rc1.executeScript(() => {
    return window.pageshowEvent.persisted;
  }));

  // Theoretically, the request should still be pending thus 0 request received.
  // However, 1 request is sent, as by default the WPT test runner, e.g.
  // content_shell in Chromium, does not enable BackgroundSync permission,
  // resulting in forcing request sending on every navigation, even if page is
  // put into BFCache.
  await expectBeacon(uuid, {count: 1});
}, `fetchLater() with activateAfter=1m sends on page entering BFCache if BackgroundSync is off.`);
