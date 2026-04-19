// Copyright (C) 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  NaNs are converted to 0 for 'count' argument to Atomics.notify
info: |
  Atomics.notify( typedArray, index, count )

  ...
  3. If count is undefined, let c be +∞.
  4. Else,
    a. Let intCount be ? ToInteger(count).
  ...

  ToInteger ( argument )

  ...
  2. If number is NaN, return +0.
  ...

includes: [nans.js]
features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

NaNs.forEach(nan => {
  assert.sameValue(Atomics.notify(i32a, 0, nan), 0, 'Atomics.notify(i32a, 0, nan) returns 0');
});
