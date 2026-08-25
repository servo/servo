// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.fill
description: >
  Absent start and end parameters are computed from initial length.
info: |
  %TypedArray%.prototype.fill ( value [ , start [ , end ] ] )

  ...
  2. Let taRecord be ? ValidateTypedArray(O, seq-cst).
  3. Let len be TypedArrayLength(taRecord).
  ...
  5. Otherwise, set value to ? ToNumber(value).
  6. Let relativeStart be ? ToIntegerOrInfinity(start).
  ...
  9. Else, let startIndex be min(relativeStart, len).
  10. If end is undefined, let relativeEnd be len; else let relativeEnd be ? ToIntegerOrInfinity(end).
  ...
  13. Else, let endIndex be min(relativeEnd, len).
  ...

features: [TypedArray, resizable-arraybuffer]
---*/

let rab = new ArrayBuffer(1, {maxByteLength: 4});
let ta = new Int8Array(rab);

let value = {
  valueOf() {
    // Set buffer to maximum byte length.
    rab.resize(4);

    // Return the fill value.
    return 123;
  }
};

// Ensure typed array is correctly initialised.
assert.sameValue(ta.length, 1);
assert.sameValue(ta[0], 0);

ta.fill(value);

// Ensure typed array has correct length and only the first element was filled.
assert.sameValue(ta.length, 4);
assert.sameValue(ta[0], 123);
assert.sameValue(ta[1], 0);
assert.sameValue(ta[2], 0);
assert.sameValue(ta[3], 0);
