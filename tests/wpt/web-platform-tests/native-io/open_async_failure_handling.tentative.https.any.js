// META: title=NativeIO API: Failures of open are properly handled.
// META: global=window,worker
// META: script=resources/support.js

'use strict';

setup(async () => {
  assert_implements(storageFoundation.open, 'storageFoundation.open is not' +
                                                ' implemented.');
});

promise_test(async testCase => {
  for (let name of kBadNativeIoNames) {
    await promise_rejects_js(
      testCase, TypeError, storageFoundation.open(name));
  }
}, 'storageFoundation.open does not allow opening files with invalid names.');
