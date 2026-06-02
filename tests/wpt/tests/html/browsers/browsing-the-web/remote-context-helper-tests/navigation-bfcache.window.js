// META: title=RemoteContextHelper navigation using BFCache
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/test-helper.js

'use strict';

promise_test(async t => {
  const rcHelper = new RemoteContextHelper();

  // Open a window with noopener so that BFCache will work.
  const rc1 = await rcHelper.addWindow(
      /*config=*/ null, /*options=*/ {features: 'noopener'});

  // Add a pageshow listener to stash the event.
  await rc1.executeScript(() => {
    window.addEventListener('pageshow', (event) => {
      window.pageshowEvent = event;
    });
  });

  // Navigate away.
  const rc2 = await rc1.navigateToNew();
  await assertSimplestScriptRuns(rc2);

  // Navigate back.
  await rc2.historyBack();

  // Verify that the document was BFCached.
  assert_true(await rc1.executeScript(() => {
    return window.pageshowEvent.persisted;
  }));
});
