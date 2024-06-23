// META: title=RemoteContextWrapper addEmbed
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/test-helper.js

'use strict';

// This tests that arguments passed to the constructor are respected.
promise_test(async t => {
  // Precondition: Test was loaded from the HTTP_ORIGIN.
  assert_equals(
      location.origin, get_host_info()['HTTP_ORIGIN'],
      'test window was loaded on HTTP_ORIGIN');

  const rcHelper = new RemoteContextHelper();

  const main = await rcHelper.addWindow();

  const headerName = 'x-wpt-test-header';
  const headerValue = 'test-escaping()';
  const iframe = await main.addEmbed(
      /*extraConfig=*/ {
        origin: 'HTTP_REMOTE_ORIGIN',
        scripts: ['/common/get-host-info.sub.js', './resources/test-script.js'],
        headers: [[headerName, headerValue]],
      },
      /*attributes=*/ {id: 'test-id'},
  );

  await assertSimplestScriptRuns(iframe);
  await assertFunctionRuns(iframe, () => testFunction(), 'testFunction exists');
  await assertOriginIsAsExpected(iframe, get_host_info()['HTTP_REMOTE_ORIGIN']);
  await assertHeaderIsAsExpected(iframe, headerName, headerValue);

  assert_equals(
      await main.executeScript(() => document.getElementById('test-id').id),
      'test-id', 'verify id');
});
