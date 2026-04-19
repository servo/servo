// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [deepEqual.js]
description: |
  pending
esid: pending
---*/
var array = [0];

var grouped = Object.groupBy(array, () => "length");

assert.deepEqual(grouped, Object.create(null, {
  length: {
    value: [0],
    writable: true,
    enumerable: true,
    configurable: true,
  },
}));

