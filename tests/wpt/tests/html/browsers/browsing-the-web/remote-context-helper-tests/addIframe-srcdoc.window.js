// META: title=RemoteContextWrapper addIframe with srcdoc
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

  const iframe = await main.addIframeSrcdoc(
      /*extraConfig=*/ {scripts: ['./resources/test-script.js']},
      /*attributes=*/ {id: 'test-id'},
  );

  await assertSimplestScriptRuns(iframe);
  await assertFunctionRuns(iframe, () => testFunction(), 'testFunction exists');

  const [id, src, srcdoc] = await main.executeScript(() => {
    const iframe = document.getElementById('test-id');
    return [iframe.id, iframe.src, iframe.srcdoc];
  });
  assert_equals(id, 'test-id', 'verify id');
  assert_equals(src, '', 'verify src');
  assert_greater_than(srcdoc.length, 0, 'verify srcdoc');
});
