// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    The 'asyncTest' helper checks that it is called with the 'async' flag.
includes: [asyncHelpers.js]
---*/
function makePromise() {
  return {
    then(res, rej) {
      // Throw a different error than Test262Error to avoid confusion about what is rejecting
      throw new Error("Should not be evaluated");
    },
  };
}
assert(
  !Object.hasOwn(globalThis, "$DONE"),
  "Without 'async' flag, $DONE should not be defined"
);
assert.throws(Test262Error, function () {
  asyncTest(makePromise);
});
