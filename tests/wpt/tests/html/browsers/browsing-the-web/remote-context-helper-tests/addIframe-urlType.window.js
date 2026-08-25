// META: title=RemoteContextWrapper addIframe
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/test-helper.js
// META: variant=?urlType=origin
// META: variant=?urlType=data
// META: variant=?urlType=blob
// META: variant=?urlType=blank

'use strict';

// This tests that arguments passed to the constructor are respected.
promise_test(async t => {
  // Precondition: Test was loaded from the HTTP_ORIGIN.
  assert_equals(
      location.origin, get_host_info()['HTTP_ORIGIN'],
      'test window was loaded on HTTP_ORIGIN');
  const rcHelper = new RemoteContextHelper();

  const main = await rcHelper.addWindow();

  const urlType = getUrlType(location);
  const iframe = await main.addIframe(
      /*extraConfig=*/ {
        scripts: ['./resources/test-script.js'],
        urlType: urlType,
      },
      /*attributes=*/ {id: 'test-id'},
  );

  await assertSimplestScriptRuns(iframe);
  await assertFunctionRuns(iframe, () => testFunction(), 'testFunction exists');

  const [id, src] = await main.executeScript(() => {
    const iframe = document.getElementById('test-id');
    return [iframe.id, iframe.src];
  });
  assert_equals(id, 'test-id', 'verify id');
  switch (urlType) {
    case 'origin':
      const url = new URL(location);
      const origin = url.origin;
      assert_equals(src.substring(0, origin.length), origin, 'verify src');
      break;
    case 'blank':
      assert_equals(src, '', 'verify src');
      break;
    case 'data':
      assert_regexp_match(src, /^data:/, 'verify src');
      break;
    case 'blob':
      assert_regexp_match(src, /^blob:/, 'verify src');
      break;
    default:
      throw new Error(`Unknown urlType: ${urlType}`);
  }
});
