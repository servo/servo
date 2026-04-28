// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.of
description: >
  Performs Set operation which ignores out-of-bounds indices.
info: |
  %TypedArray%.of ( ...items )

  ...
  6. Repeat, while k < len,
    ...
    c. Perform ? Set(newObj, Pk, kValue, true).
    ...

features: [TypedArray, resizable-arraybuffer]
---*/

let rab = new ArrayBuffer(3, {maxByteLength: 4});
let ta = new Int8Array(rab);

let one = {
  valueOf() {
    // Shrink buffer. Assignment to `ta[0]` should be ignored.
    rab.resize(0);
    return 1;
  }
};

let two = {
  valueOf() {
    // Grow buffer. All following assignment should succeed.
    rab.resize(4);
    return 2;
  }
};

// Typed array is correctly zero initialised.
assert.sameValue(ta.length, 3);
assert.sameValue(ta[0], 0);
assert.sameValue(ta[1], 0);
assert.sameValue(ta[2], 0);

let result = Int8Array.of.call(function() {
  return ta;
}, one, two, 3);

// Correct result value.
assert.sameValue(result, ta);

// Values are correctly set.
assert.sameValue(ta.length, 4);
assert.sameValue(ta[0], 0);
assert.sameValue(ta[1], 2);
assert.sameValue(ta[2], 3);
assert.sameValue(ta[3], 0);
