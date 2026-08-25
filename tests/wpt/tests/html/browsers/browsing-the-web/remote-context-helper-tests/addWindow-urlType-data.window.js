// META: title=RemoteContextHelper addWindow with extra config
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/test-helper.js

'use strict';

// This tests that urlType of "data" fails for a new window.
promise_test(async t => {
  const urlType = getUrlType(location);

  const rcHelper = new RemoteContextHelper();

  return promise_rejects_js(
      t, TypeError,
      rcHelper.addWindow(
          /*extraConfig=*/ {
            urlType: 'data',
          },
          ),
      'creating window with data: URL fails.');
});
