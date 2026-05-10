// Copyright 2019 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    object values compare correctly.
includes: [deepEqual.js]
---*/


assert.deepEqual({}, {});
assert.deepEqual({ a: 1, b: true }, { a: 1, b: true });

assert.throws(Test262Error, function () { assert.deepEqual({}, { a: 1, b: true }); });
assert.throws(Test262Error, function () { assert.deepEqual({ a: 1, b: true }, { a: 1, b: false }); });
