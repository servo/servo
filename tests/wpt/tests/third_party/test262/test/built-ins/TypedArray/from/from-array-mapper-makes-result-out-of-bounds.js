// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.from
description: >
  If the mapper function makes result typed array out-of-bounds, .from performs Set operation which ignores out-of-bounds indices.
info: |
  %TypedArray%.from ( source [ , mapfn [ , thisArg ] ] )

  ...
  12. Repeat, while k < len,
    ...
    c. If mapping is true, then
      i. Let mappedValue be ? Call(mapfn, thisArg, « kValue, 𝔽(k) »).
    ...
    e. Perform ? Set(targetObj, Pk, mappedValue, true).
    ...
features: [TypedArray, resizable-arraybuffer]
---*/

let rab = new ArrayBuffer(3, {maxByteLength: 5});
let target = new Int8Array(rab);
let values = [0, 1, 2];

let result = Int32Array.from.call(function() {
  return target;
}, values, v => {
  if (v === 1) {
    rab.resize(1);
  }
  return v + 10;
});

assert.sameValue(result, target);
assert.sameValue(result.length, 1);
assert.sameValue(result[0], 10);
