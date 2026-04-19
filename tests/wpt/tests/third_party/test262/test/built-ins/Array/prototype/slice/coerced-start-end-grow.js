// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.slice
description: >
  Array.p.slice behaves correctly on TypedArrays backed by resizable buffers that
  are grown by argument coercion.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// The start argument grows the resizable array buffer rab.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab);
  for (let i = 0; i < 4; ++i) {
    lengthTracking[i] = MayNeedBigInt(lengthTracking, i + 1);
  }
  const evil = {
    valueOf: () => {
      rab.resize(6 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  assert.compareArray(ToNumbers(Array.prototype.slice.call(lengthTracking, evil)), [
    1,
    2,
    3,
    4
  ]);
  assert.sameValue(rab.byteLength, 6 * ctor.BYTES_PER_ELEMENT);
}

// The end argument grows the resizable array buffer rab.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab);
  for (let i = 0; i < 4; ++i) {
    lengthTracking[i] = MayNeedBigInt(lengthTracking, i + 1);
  }

  const evil = {
    valueOf: () => {
      rab.resize(6 * ctor.BYTES_PER_ELEMENT);
      return 5;
    }
  };
  assert.compareArray(ToNumbers(Array.prototype.slice.call(lengthTracking,4,evil)), [
  ]);
  assert.compareArray(ToNumbers(Array.prototype.slice.call(lengthTracking,3,evil)), [
    4,
    0
  ]);
  assert.sameValue(rab.byteLength, 6 * ctor.BYTES_PER_ELEMENT);
}
