// Copyright 2019 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  map/set values compare correctly.
includes: [deepEqual.js]
---*/

assert.deepEqual(new Set(), new Set());
assert.deepEqual(new Set([1, "a", true]), new Set([1, "a", true]));
assert.deepEqual(new Map(), new Map());
assert.deepEqual(new Map([[1, "a"], ["b", true]]), new Map([[1, "a"], ["b", true]]));

assert.throws(Test262Error, function () { assert.deepEqual(new Set([]), new Set([1])); });
assert.throws(Test262Error, function () { assert.deepEqual(new Set([1, "a", true]), new Set([1, "a", false])); });
assert.throws(Test262Error, function () { assert.deepEqual(new Map([]), new Map([[1, "a"], ["b", true]])); });
assert.throws(Test262Error, function () { assert.deepEqual(new Map([[1, "a"], ["b", true]]), new Map([[1, "a"], ["b", false]])); });
assert.throws(Test262Error, function () { assert.deepEqual(new Map([[1, "a"], ["b", true]]), new Set([[1, "a"], ["b", false]])); });
