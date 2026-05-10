// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.join
description: >
  ToString is called once when the array is resized.
info: |
  %TypedArray%.prototype.join ( separator )

  ...
  2. Let taRecord be ? ValidateTypedArray(O, seq-cst).
  3. Let len be TypedArrayLength(taRecord).
  ...
  5. Else, let sep be ? ToString(separator).
  ...

features: [TypedArray, resizable-arraybuffer]
---*/

let rab = new ArrayBuffer(3, {maxByteLength: 5});
let ta = new Int8Array(rab);

let callCount = 0;

let index = {
  toString() {
    callCount++;
    rab.resize(0);
    return "-";
  }
};

assert.sameValue(callCount, 0);

let r = ta.join(index);

assert.sameValue(callCount, 1);
assert.sameValue(r, "--");
