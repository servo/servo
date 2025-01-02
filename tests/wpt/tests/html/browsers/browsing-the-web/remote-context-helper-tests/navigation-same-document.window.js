// META: title=RemoteContextHelper navigation using BFCache
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/test-helper.js

'use strict';

async function assertLocationIs(remoteContextWrapper, expectedLocation) {
  assert_equals(
      await remoteContextWrapper.executeScript(() => {
        return location.toString();
      }),
      expectedLocation, 'verify location');
}

promise_test(async t => {
  const rcHelper = new RemoteContextHelper();

  const rc = await rcHelper.addWindow();

  const oldLocation = await rc.executeScript(() => {
    return location.toString();
  });
  const newLocation = oldLocation + '#fragment';

  // Navigate to same document.
  await rc.navigateTo(newLocation);

  // Verify that the window navigated.
  await assertLocationIs(rc, newLocation);

  // Navigate back.
  await rc.historyBack(oldLocation);

  // Verify that the window navigated back and the executor is running.
  await assertLocationIs(rc, oldLocation);
});
