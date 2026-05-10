// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    The 'asyncTest' helper reports synchronous errors via $DONE.
includes: [asyncHelpers.js]
---*/
var called = false;
var msg = "Should not be rethrown";
function $DONE(error) {
  called = true;
  assert(error instanceof Test262Error);
  assert.sameValue(error.message, msg, "Should report correct error");
}
asyncTest(function () {
  throw new Test262Error(msg);
});
assert(called, "asyncTest called $DONE with a synchronously thrown error");
