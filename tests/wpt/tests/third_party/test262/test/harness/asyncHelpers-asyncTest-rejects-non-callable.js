// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    The 'asyncTest' helper rejects non-callable test functions.
includes: [asyncHelpers.js, compareArray.js]
---*/

const doneValues = [];
function $DONE(error) {
  doneValues.push(error instanceof Test262Error);
}
asyncTest(null);
asyncTest({});
asyncTest("string");
asyncTest(42);
asyncTest(undefined);
asyncTest();
assert.compareArray(doneValues, [true, true, true, true, true, true]);
