// META: title=RemoteContextWrapper navigateToNew
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/test-helper.js

'use strict';

promise_test(async t => {
  // Precondition: Test was loaded from the HTTP_ORIGIN.
  assert_equals(
      location.origin, get_host_info()['HTTP_ORIGIN'],
      'test window was loaded on HTTP_ORIGIN');

  const rcHelper = new RemoteContextHelper();

  const main = await rcHelper.addWindow();

  const headerName = 'x-wpt-test-header';
  const headerValue = 'test-escaping()';
  const newMain = await main.navigateToNew(
      {
        origin: 'HTTP_REMOTE_ORIGIN',
        scripts: ['/common/get-host-info.sub.js', './resources/test-script.js'],
        headers: [[headerName, headerValue]],
      },
  );

  await assertSimplestScriptRuns(newMain);
  await assertFunctionRuns(
      newMain, () => testFunction(), 'testFunction exists');

  const remoteOrigin = get_host_info()['HTTP_REMOTE_ORIGIN'];
  await assertOriginIsAsExpected(newMain, remoteOrigin);
  await assertHeaderIsAsExpected(newMain, headerName, headerValue);
});
