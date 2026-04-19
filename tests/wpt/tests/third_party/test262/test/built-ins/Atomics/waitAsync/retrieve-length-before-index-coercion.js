// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  TypedArray length is retrieved before index parameter coercion.
info: |
  25.4.14 Atomics.waitAsync ( typedArray, index, value, timeout )
    1. Return ? DoWait(async, typedArray, index, value, timeout).

  25.4.3.14 DoWait ( mode, typedArray, index, value, timeout )
    ...
    4. Let i be ? ValidateAtomicAccess(taRecord, index).
    ...

  25.4.3.2 ValidateAtomicAccess ( taRecord, requestIndex )
    1. Let length be TypedArrayLength(taRecord).
    2. Let accessIndex be ? ToIndex(requestIndex).
    3. Assert: accessIndex ≥ 0.
    4. If accessIndex ≥ length, throw a RangeError exception.
    ...
features: [Atomics.waitAsync, Atomics, TypedArray, resizable-arraybuffer]
---*/

var gsab = new SharedArrayBuffer(0, {maxByteLength: 4});
var ta = new Int32Array(gsab);

var index = {
  valueOf() {
    gsab.grow(4);
    return 0;
  }
};

var value = {
  valueOf() {
    throw new Test262Error("Unexpected value coercion");
  }
};

var timeout = {
  valueOf() {
    throw new Test262Error("Unexpected timeout coercion");
  }
};

assert.throws(RangeError, function() {
  Atomics.waitAsync(ta, index, value, timeout);
});

assert.sameValue(gsab.byteLength, 4);
