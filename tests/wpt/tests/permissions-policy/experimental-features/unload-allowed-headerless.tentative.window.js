// META: title='unload' Policy : allowed in headerless doc when allowed in main frame.
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/unload-helper.js
// META: variant=?urlType=srcdoc
// META: variant=?urlType=data
// META: variant=?urlType=blob
// META: variant=?urlType=blank

'use strict';

promise_test(async t => {
  const rcHelper = new RemoteContextHelper({
    scripts: ['./resources/unload-helper.js'],
  });
  // In the same browsing context group to ensure BFCache is not used.
  const main = await rcHelper.addWindow(
      {headers: [['Permissions-Policy', 'unload=*']]},
  );

  const url = new URL(location);
  const urlType = url.searchParams.get('urlType');

  if (urlType == 'srcdoc') {
    const subframe = await main.addIframeSrcdoc(
        /*extraConfig=*/ {}, /*attributes=*/ {allow: 'unload'});
    await assertWindowRunsUnload(subframe, 'subframe', {shouldRunUnload: true});
  } else {
    const subframe = await main.addIframe(
        /*extraConfig=*/ {urlType: urlType}, /*attributes=*/ {allow: 'unload'});
    // The other URL types cannot easily test unload directly. `data: and
    // `blob:` documents cannot access storage. `about:blank` loses the content
    // that was written into it when reloaded on going back and so stops
    // functioning as a remote execution context.
    await assertWindowAllowsUnload(
        subframe, 'subframe', {shouldRunUnload: true});
  }

  await assertWindowRunsUnload(main, 'main', {shouldRunUnload: true});
});
