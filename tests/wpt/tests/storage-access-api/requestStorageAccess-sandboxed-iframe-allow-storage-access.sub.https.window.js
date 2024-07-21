// META: script=helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

test(() => {
  let iframe = document.createElement('iframe');
  assert_true(iframe.sandbox.supports('allow-storage-access-by-user-activation'), '`allow-storage-access-by-user-activation`' +
    'sandbox attribute should be supported');
}, "`allow-storage-access-by-user-activation` sandbox attribute is supported");

(async function () {
  const frameSourceUrl = 'https://{{hosts[alt][www]}}:{{ports[https][0]}}/storage-access-api/requestStorageAccess-sandboxed-iframe.sub.https.window.html';

  let sandboxAttribute =
    'allow-scripts allow-same-origin';
  let testCase = 'sandboxed-iframe';

  RunTestsInIFrame(
    frameSourceUrl + `?testCase=${testCase}`,
    sandboxAttribute);

  sandboxAttribute += ' allow-storage-access-by-user-activation';
  testCase = 'sandboxed-iframe-allow-storage-access-by-user-activation';

  RunTestsInIFrame(
    frameSourceUrl + `?testCase=${testCase}`,
    sandboxAttribute);
})();
