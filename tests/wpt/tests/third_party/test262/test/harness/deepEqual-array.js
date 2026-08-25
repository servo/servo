// Copyright 2019 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  array values compare correctly.
includes: [deepEqual.js]
---*/

assert.deepEqual([], []);
assert.deepEqual([1, "a", true], [1, "a", true]);

assert.throws(Test262Error, function () { assert.deepEqual([], [1]); });
assert.throws(Test262Error, function () { assert.deepEqual([1, "a", true], [1, "a", false]); });
