// META: title=NativeIO API: Interface is not exposed in untrustworthy origin.
// META: global=window,dedicatedworker

'use strict';

test(testCase => {
  var present = (typeof storageFoundation !== 'undefined');
  assert_false(present);
}, 'NativeIO should not be accessible from an untrustworthy origin');
