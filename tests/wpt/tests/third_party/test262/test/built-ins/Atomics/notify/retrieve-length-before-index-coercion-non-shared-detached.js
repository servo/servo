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
features: [Atomics, TypedArray]
includes: [detachArrayBuffer.js]
---*/

var ab = new ArrayBuffer(4);
var ta = new Int32Array(ab);

var index = {
  valueOf() {
    $DETACHBUFFER(ab);
    return 0;
  }
};

assert.sameValue(Atomics.notify(ta, index), 0);

assert.sameValue(ab.byteLength, 0);
