// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js
// META: script=/fetch/fetch-later/resources/fetch-later-helper.js

'use strict';

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);

  // Loads an iframe that creates a fetchLater request w/ short timeout.
  const iframe = await loadScriptAsIframe(`
    fetchLater("${url}", {activateAfter: 1000});  // 1s
  `);
  // Deletes the iframe to trigger deferred request sending.
  document.body.removeChild(iframe);

  // The iframe should have sent all requests.
  await expectBeacon(uuid, {count: 1});
}, 'fetchLater() sends out based on activateAfter.');

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);
  // Sets no option to test the default behavior when a document enters BFCache.
  const helper = new RemoteContextHelper();
  // Opens a window with noopener so that BFCache will work.
  const rc1 = await helper.addWindow(
      /*config=*/ null, /*options=*/ {features: 'noopener'});

  // Creates a fetchLater request with short timeout. It should be sent out
  // even if the document is then put into BFCache.
  await rc1.executeScript(url => {
    fetchLater(url, {activateAfter: 1000});  // 1s.
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

  await expectBeacon(uuid, {count: 1});
}, 'fetchLater() sends out based on activateAfter, even if document is in BFCache.');
