// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [detachArrayBuffer.js]
description: |
  pending
esid: pending
---*/
// Ensure %TypedArray%.prototype.fill checks for detached buffers.

function DetachArrayBufferValue(buffer, value) {
    return {
        valueOf() {
            $DETACHBUFFER(buffer);
            return value;
        }
    };
}

function DetachTypedArrayValue(ta, value) {
    return {
        valueOf() {
            $DETACHBUFFER(ta.buffer);
            return value;
        }
    };
}

// Test when ArrayBuffer is already reified.
for (let length of [0, 1, 10, 4096]) {
    let ta = new Int32Array(length);
    let value = DetachArrayBufferValue(ta.buffer, 123);
    assert.throws(TypeError, () => ta.fill(value));
}

// Test when ArrayBuffer is reified during the fill() call.
for (let length of [0, 1, 10, 4096]) {
    let ta = new Int32Array(length);
    let value = DetachTypedArrayValue(ta, 123);
    assert.throws(TypeError, () => ta.fill(value));
}

