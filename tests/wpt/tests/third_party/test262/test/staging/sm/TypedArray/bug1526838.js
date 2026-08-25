// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  pending
esid: pending
---*/
const testArray = [1n];
for (const constructor of anyTypedArrayConstructors) {
    assert.throws(TypeError, () => new constructor(testArray));
    assert.throws(TypeError, () => new constructor(testArray.values()));
}

