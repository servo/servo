// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [detachArrayBuffer.js]
description: |
  pending
esid: pending
---*/

class Err {}

const indices = [
  -Infinity, -10, -0.5, -0, 0, 0.5, 10, Infinity, NaN
];

let value = {
  valueOf() {
    throw new Err;
  }
};

let ta = new Int32Array(5);
for (let index of indices) {
  assert.throws(Err, () => ta.with(index, value), Err);
}

for (let index of indices) {
  let ta = new Int32Array(5);

  let value = {
    valueOf() {
      $DETACHBUFFER(ta.buffer);
      return 0;
    }
  };

  assert.throws(RangeError, () => ta.with(index, value));
}

