// META: title=RemoteContextWrapper addIframe with srcdoc and startOn
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/test-helper.js

'use strict';

promise_test(async t => {
  const rcHelper = new RemoteContextHelper();

  const main = await rcHelper.addWindow();

  const iframe = await main.addIframeSrcdoc(
      /*extraConfig=*/ {startOn: 'pageshow'});

  await assertSimplestScriptRuns(iframe);
  await assert_equals(
      await iframe.executeScript(() => {
        return executorStartEvent.type;
      }),
      'pageshow');
});
