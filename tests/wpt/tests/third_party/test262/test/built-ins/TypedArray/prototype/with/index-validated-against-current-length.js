// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.with
description: >
  The index is validated against the current length.
info: |
  %TypedArray%.prototype.with ( index, value )

  1. Let O be the this value.
  2. Let taRecord be ? ValidateTypedArray(O, SEQ-CST).
  3. Let len be TypedArrayLength(taRecord).
  ...
  8. Else, let numericValue be ? ToNumber(value).
  ...
  10. Let A be ? TypedArrayCreateSameType(O, « 𝔽(len) »).
  ...
  13. Return A.

features: [TypedArray, resizable-arraybuffer]
---*/

let rab = new ArrayBuffer(2, {maxByteLength: 5});
let ta = new Int8Array(rab);

ta[0] = 11;
ta[1] = 22;

// Ensure typed array is correctly initialised.
assert.sameValue(ta.length, 2);
assert.sameValue(ta[0], 11);
assert.sameValue(ta[1], 22);

// Index is initially out-of-bounds.
let index = 4;

let value = {
  valueOf() {
    rab.resize(5);
    return 123;
  }
};

let result = ta.with(index, value);

// Typed array has been resized.
assert.sameValue(ta.length, 5);
assert.sameValue(ta[0], 11);
assert.sameValue(ta[1], 22);
assert.sameValue(ta[2], 0);
assert.sameValue(ta[3], 0);
assert.sameValue(ta[4], 0);

// Result is correctly initialised.
assert.sameValue(result.length, 2);
assert.sameValue(result[0], 11);
assert.sameValue(result[1], 22);
