// META: title=RemoteContextHelper addWindow with extra config
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/test-helper.js

'use strict';

// This tests that arguments passed to the constructor are respected.
promise_test(async t => {
  const header1Name = 'x-wpt-test-header1';
  const header1Value = 'test-escaping1()';
  const rcHelper = new RemoteContextHelper({
    origin: 'HTTP_REMOTE_ORIGIN',
    scripts: ['/common/get-host-info.sub.js', './resources/test-script.js'],
    headers: [[header1Name, header1Value]],
  });

  const header2Name = 'x-wpt-test-header2';
  const header2Value = 'test-escaping2()';
  const main = await rcHelper.addWindow(
      {
        origin: location.origin,
        scripts: [new URL('./resources/test-script2.js', location).toString()],
        headers: [[header2Name, header2Value]],
      },
  );

  await assertSimplestScriptRuns(main);
  await assertFunctionRuns(main, () => testFunction(), 'testFunction exists');
  await assertFunctionRuns(main, () => testFunction2(), 'testFunction2 exists');
  await assertOriginIsAsExpected(main, location.origin);
  await assertHeaderIsAsExpected(main, header1Name, header1Value);
  await assertHeaderIsAsExpected(main, header2Name, header2Value);
});
