// META: title=RemoteContextWrapper addScripts
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/test-helper.js

'use strict';

// This tests that arguments passed to the constructor are respected.
promise_test(async t => {
  const rcHelper = new RemoteContextHelper();

  const main = await rcHelper.addWindow();
  await assertSimplestScriptRuns(main);

  await main.addScripts(['./resources/test-script.js']);
  await assertFunctionRuns(main, () => testFunction(), 'testFunction exists');
});
