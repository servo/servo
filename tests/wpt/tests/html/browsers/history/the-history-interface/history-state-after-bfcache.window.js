// META: title=Navigating back to a bfcached page does not reset history.state
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js

// See https://github.com/whatwg/html/issues/6652.

'use strict';

promise_test(async t => {
  const rcHelper = new RemoteContextHelper();

  // Open a window with noopener so that BFCache will work.
  const rc = await rcHelper.addWindow(null, { features: "noopener" });

  // Add a pageshow listener to stash the event, and set history.state using replaceState().
  await rc.executeScript(() => {
    window.addEventListener('pageshow', (event) => {
      window.pageshowEvent = event;
    });

    history.replaceState({ foo: 'bar' }, '', '');
    window.stashedHistoryState = history.state;
  });

  const rc2 = await rc.navigateToNew();
  await rc2.historyBack();

  assert_implements_optional(
    await rc.executeScript(() => window.pageshowEvent.persisted),
    'precondition: document was bfcached'
  );

  assert_equals(
    await rc.executeScript(() => history.state.foo),
    'bar',
    'history.state was restored correctly'
  );

  assert_true(
    await rc.executeScript(() => window.stashedHistoryState === history.state),
    'history.state did not get serialized and deserialized');
});
