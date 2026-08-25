// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.set
description: >
  TypedArray.p.set behaves correctly on TypedArrays backed by a
  resizable buffer is grown due to the source's length getter
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// Orig. array: [0, 2, 4, 6]
//              [0, 2, 4, 6] << fixedLength
//                    [4, 6] << fixedLengthWithOffset
//              [0, 2, 4, 6, ...] << lengthTracking
//                    [4, 6, ...] << lengthTrackingWithOffset
function CreateSourceProxy(length, rab, resizeTo) {
  return new Proxy({}, {
    get(target, prop, receiver) {
      if (prop == 'length') {
        rab.resize(resizeTo);
        return length;
      }
      return true; // Can be converted to both BigInt and Number.
    }
  });
}

// Test that we still throw for lengthTracking TAs if the source length is
// too large, even though we resized in the length getter (we check against
// the original length).
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab, 0);
  const resizeTo = 6 * ctor.BYTES_PER_ELEMENT;
  assert.throws(RangeError, () => {
    lengthTracking.set(CreateSourceProxy(6, rab, resizeTo));
  });
  assert.compareArray(ToNumbers(new ctor(rab)), [
    0,
    2,
    4,
    6,
    0,
    0
  ]);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  const resizeTo = 6 * ctor.BYTES_PER_ELEMENT;
  assert.throws(RangeError, () => {
    lengthTrackingWithOffset.set(CreateSourceProxy(4, rab, resizeTo));
  });
  assert.compareArray(ToNumbers(new ctor(rab)), [
    0,
    2,
    4,
    6,
    0,
    0
  ]);
}
