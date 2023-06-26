// META: title='unload' Policy : disallowed when header is ()
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/unload-helper.js
// META: timeout=long

'use strict';

// Check that unload can be disabled by policy in main frame and subframe.
promise_test(async t => {
  const rcHelper =
      new RemoteContextHelper({scripts: ['./resources/unload-helper.js']});
  // In the same browsing context group to ensure BFCache is not used.
  const main = await rcHelper.addWindow(
      {headers: [['Permissions-Policy', 'unload=()']]},
  );
  const subframe = await main.addIframe();
  await assertWindowRunsUnload(subframe, 'subframe', {shouldRunUnload: false});
  await assertWindowRunsUnload(main, 'main', {shouldRunUnload: false});
});
