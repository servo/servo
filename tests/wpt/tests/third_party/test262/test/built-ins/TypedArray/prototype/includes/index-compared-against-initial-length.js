// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.includes
description: >
  Index is compared against the initial length.
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
let ta = new Int8Array(rab);

let index = {
  valueOf() {
    // Shrink buffer to zero.
    rab.resize(0);

    // Index is larger than the initial length.
    return 10;
  }
};

// Auto-length is correctly tracked.
assert.sameValue(ta.length, 4);

let result = ta.includes(undefined, index);

// Auto-length is correctly set to zero.
assert.sameValue(ta.length, 0);

assert.sameValue(result, false);
