// META: title=Synchronous NativeIO API: Failures of open are properly handled.
// META: global=dedicatedworker
// META: script=resources/support.js

'use strict';

setup(() => {
  assert_implements(storageFoundation.openSync, 'storageFoundation.openSync' +
                                                ' is not implemented.');
});

test(testCase => {
  for (let name of kBadNativeIoNames) {
    assert_throws_js(TypeError, () => {storageFoundation.openSync(name)});
  }
}, 'storageFoundation.openSync does not allow opening files with invalid ' +
     'names.');
