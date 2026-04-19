// Copyright (C) 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  Return abrupt when ToInteger throws an exception on 'count' argument to Atomics.notify
info: |
  Atomics.notify( typedArray, index, count )

  ...
  3. If count is undefined, let c be +∞.
  4. Else,
    a. Let intCount be ? ToInteger(count).
  ...

features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

const poisoned = {
  valueOf: function() {
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(Test262Error, function() {
  Atomics.notify(i32a, 0, poisoned);
});
