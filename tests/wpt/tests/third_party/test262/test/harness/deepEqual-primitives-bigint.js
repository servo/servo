// Copyright 2019 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    primitive BigInt values compare correctly.
features: [BigInt]
includes: [deepEqual.js]
---*/

assert.deepEqual(1n, 1n);
assert.deepEqual(Object(1n), 1n);

assert.throws(Test262Error, function () { assert.deepEqual(1n, 1); });
assert.throws(Test262Error, function () { assert.deepEqual(1n, 2n); });
