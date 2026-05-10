// Copyright 2019 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    primitive values compare correctly.
includes: [deepEqual.js]
---*/

/// <reference path="../../harness/assert.js" />
/// <reference path="../../harness/deepEqual.js" />

var s1 = Symbol();
var s2 = Symbol();
assert.deepEqual(null, null);
assert.deepEqual(undefined, undefined);
assert.deepEqual("a", "a");
assert.deepEqual(1, 1);
assert.deepEqual(true, true);
assert.deepEqual(s1, s1);
assert.deepEqual(Object("a"), "a");
assert.deepEqual(Object(1), 1);
assert.deepEqual(Object(true), true);
assert.deepEqual(Object(s1), s1);

assert.throws(Test262Error, function () { assert.deepEqual(null, 0); });
assert.throws(Test262Error, function () { assert.deepEqual(undefined, 0); });
assert.throws(Test262Error, function () { assert.deepEqual("", 0); });
assert.throws(Test262Error, function () { assert.deepEqual("1", 1); });
assert.throws(Test262Error, function () { assert.deepEqual("1", "2"); });
assert.throws(Test262Error, function () { assert.deepEqual(true, 1); });
assert.throws(Test262Error, function () { assert.deepEqual(true, false); });
assert.throws(Test262Error, function () { assert.deepEqual(s1, "Symbol()"); });
assert.throws(Test262Error, function () { assert.deepEqual(s1, s2); });
