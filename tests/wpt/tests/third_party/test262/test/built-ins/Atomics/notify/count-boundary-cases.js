// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  Allowed boundary cases for 'count' argument to Atomics.notify
info: |
  Atomics.notify( typedArray, index, count )

  ...
  3. If count is undefined, let c be +∞.
  4. Else,
    a. Let intCount be ? ToInteger(count).
  ...

  ToInteger ( argument )

  1. Let number be ? ToNumber(argument).
  2. If number is NaN, return +0.
  3. If number is +0, -0, +∞, or -∞, return number.
  4. Return the number value that is the same sign as number
      and whose magnitude is floor(abs(number)).

features: [Atomics, SharedArrayBuffer, TypedArray]
---*/

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

assert.sameValue(
  Atomics.notify(i32a, 0, -3),
  0,
  'Atomics.notify(i32a, 0, -3) returns 0'
);
assert.sameValue(
  Atomics.notify(i32a, 0, Number.POSITIVE_INFINITY),
  0,
  'Atomics.notify(i32a, 0, Number.POSITIVE_INFINITY) returns 0'
);
assert.sameValue(
  Atomics.notify(i32a, 0, undefined),
  0,
  'Atomics.notify(i32a, 0, undefined) returns 0'
);
assert.sameValue(
  Atomics.notify(i32a, 0, '33'),
  0,
  'Atomics.notify(i32a, 0, \'33\') returns 0'
);
assert.sameValue(
  Atomics.notify(i32a, 0, { valueOf: 8 }),
  0,
  'Atomics.notify(i32a, 0, {valueOf: 8}) returns 0'
);
assert.sameValue(
  Atomics.notify(i32a, 0),
  0,
  'Atomics.notify(i32a, 0) returns 0'
);
