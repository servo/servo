// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.includes
description: >
  Index is compared against the initial length when typed array is made out-of-bounds.
info: |
  %TypedArray%.prototype.includes ( searchElement [ , fromIndex ] )

  ...
  2. Let taRecord be ? ValidateTypedArray(O, seq-cst).
  3. Let len be TypedArrayLength(taRecord).
  ...
  5. Let n be ? ToIntegerOrInfinity(fromIndex).
  ...
  9. If n ≥ 0, then
    a. Let k be n.
  ...
  11. Repeat, while k < len,
    ...

features: [TypedArray, resizable-arraybuffer]
---*/

let rab = new ArrayBuffer(4, {maxByteLength: 20});

// Uses byteOffset to make typed array out-of-bounds when shrinking size to zero.
let byteOffset = 1;

let ta = new Int8Array(rab, byteOffset);

let index = {
  valueOf() {
    // Shrink buffer to zero.
    rab.resize(0);

    // Index is larger than the initial length.
    return 10;
  }
};

// Typed array is in-bounds.
assert.sameValue(ta.length, 3);

let result = ta.includes(undefined, index);

// Typed array is out-of-bounds.
assert.sameValue(ta.length, 0);

assert.sameValue(result, false);
