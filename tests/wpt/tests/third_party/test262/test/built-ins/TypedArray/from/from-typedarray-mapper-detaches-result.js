// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.from
description: >
  If the mapper function detaches the result typed array, .from performs Set operation which ignores out-of-bounds indices.
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
includes: [detachArrayBuffer.js]
features: [TypedArray]
---*/

let ab = new ArrayBuffer(3);
let target = new Int8Array(ab);
let values = new Int8Array([0, 1, 2]);

let result = Int32Array.from.call(function() {
  return target;
}, values, v => {
  if (v === 1) {
    $DETACHBUFFER(ab);
  }
  return v + 10;
});

assert.sameValue(result, target);
assert.sameValue(result.length, 0);
