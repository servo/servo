// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  TypedArray length is retrieved before index parameter coercion.
info: |
  25.4.15 Atomics.notify ( typedArray, index, count )
    ...
    2. Let byteIndexInBuffer be ? ValidateAtomicAccess(taRecord, index).
    ...

  25.4.3.2 ValidateAtomicAccess ( taRecord, requestIndex )
    1. Let length be TypedArrayLength(taRecord).
    2. Let accessIndex be ? ToIndex(requestIndex).
    3. Assert: accessIndex ≥ 0.
    4. If accessIndex ≥ length, throw a RangeError exception.
    ...
features: [Atomics, TypedArray, resizable-arraybuffer]
---*/

var rab = new ArrayBuffer(0, {maxByteLength: 4});
var ta = new Int32Array(rab);

var index = {
  valueOf() {
    rab.resize(4);
    return 0;
  }
};

var count = {
  valueOf() {
    throw new Test262Error("Unexpected count coercion");
  }
};

assert.throws(RangeError, function() {
  Atomics.notify(ta, index, count);
});

assert.sameValue(rab.byteLength, 4);
