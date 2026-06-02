// META: title=RemoteContextWrapper addWorker no global variable
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/test-helper.js

'use strict';

// This tests that arguments passed to the constructor are respected.
// Specifically `null` for the parameter `globalVariable`.
promise_test(async t => {
  const rcHelper = new RemoteContextHelper();

  const main = await rcHelper.addWindow();

  const headerName = 'x-wpt-test-header';
  const headerValue = 'test-escaping()';
  const worker = await main.addWorker(
    null,
      {
        scripts: ['/common/get-host-info.sub.js', './resources/test-script.js'],
        headers: [[headerName, headerValue]],
      },
  );

  await assertSimplestScriptRuns(worker);
  await assertFunctionRuns(worker, () => testFunction(), 'testFunction exists');
  await assertOriginIsAsExpected(worker, location.origin);
  await assertHeaderIsAsExpected(worker, headerName, headerValue);
});
