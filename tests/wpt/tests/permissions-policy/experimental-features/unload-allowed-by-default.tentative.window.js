// META: title='unload' Policy : allowed by default
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/unload-helper.js
// META: timeout=long

'use strict';

// Check that unload is allowed by policy in main frame and subframe by default.
promise_test(async t => {
  const rcHelper =
      new RemoteContextHelper({scripts: ['./resources/unload-helper.js']});
  // In the same browsing context group to ensure BFCache is not used.
  const main = await rcHelper.addWindow();
  const sameOriginSubframe = await main.addIframe();
  const crossOriginSubframe = await main.addIframe({ origin: 'HTTP_REMOTE_ORIGIN' });
  await assertWindowRunsUnload(sameOriginSubframe, 'sameOriginSubframe', { shouldRunUnload: true });
  await assertWindowRunsUnload(crossOriginSubframe, 'crossOriginSubframe', { shouldRunUnload: true });
  await assertWindowRunsUnload(main, 'main', {shouldRunUnload: true});
});
