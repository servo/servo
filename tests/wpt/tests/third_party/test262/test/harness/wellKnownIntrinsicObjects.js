// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Basic tests for getWellKnownIntrinsicObject harness function
includes: [wellKnownIntrinsicObjects.js]
---*/

// Accessible in every implementation
var intrinsicArray = getWellKnownIntrinsicObject('%Array%');
assert(Object.is(Array, intrinsicArray));

assert.throws(Test262Error, function () {
  // Exists but is not accessible in any implementation
  getWellKnownIntrinsicObject('%AsyncFromSyncIteratorPrototype%');
});

assert.throws(Test262Error, function () {
  // Does not exist in any implementation
  getWellKnownIntrinsicObject('%NotSoWellKnownIntrinsicObject%');
});
