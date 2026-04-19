// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    The 'asyncTest' helper rejects test functions that do not return a thenable.
includes: [asyncHelpers.js, compareArray.js]
---*/

const doneValues = [];
function $DONE(error) {
  // Will be a TypeError from trying to invoke .then() on non-thenable
  doneValues.push(error instanceof TypeError);
}
asyncTest(function () {
  return null;
});
asyncTest(function () {
  return {};
});
asyncTest(function () {
  return "string";
});
asyncTest(function () {
  return 42;
});
asyncTest(function () {});
asyncTest(function () {
  return function () {};
});
assert.compareArray(doneValues, [true, true, true, true, true, true]);
