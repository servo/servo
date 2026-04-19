// Copyright 2023 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.slice
description: >
  Array.p.slice behaves correctly on TypedArrays backed by resizable buffers that
  are shrunk by argument coercion.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// The start argument shrinks the resizable array buffer rab.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  assert.compareArray(ToNumbers(Array.prototype.slice.call(fixedLength, evil)), [
    undefined,
    undefined,
    undefined,
    undefined
  ]);
  assert.compareArray(ToNumbers(Array.prototype.slice.call(fixedLength, evil)), [
  ]);
  assert.sameValue(rab.byteLength, 2 * ctor.BYTES_PER_ELEMENT);
}
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab);
  for (let i = 0; i < 4; ++i) {
    lengthTracking[i] = MayNeedBigInt(lengthTracking, i + 1);
  }
  const evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 0;
    }
  };
  assert.compareArray(ToNumbers(Array.prototype.slice.call(lengthTracking, evil)), [
    1,
    2,
    undefined,
    undefined
  ]);
  assert.compareArray(ToNumbers(Array.prototype.slice.call(lengthTracking, evil)), [
    1,
    2
  ]);
  assert.sameValue(rab.byteLength, 2 * ctor.BYTES_PER_ELEMENT);
}

// The end argument shrinks the resizable array buffer rab.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 3;
    }
  };
  assert.compareArray(ToNumbers(Array.prototype.slice.call(fixedLength, 2, evil)), [
    undefined
  ]);
  assert.compareArray(ToNumbers(Array.prototype.slice.call(fixedLength, 2, evil)), [
  ]);
  assert.sameValue(rab.byteLength, 2 * ctor.BYTES_PER_ELEMENT);
}
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab);
  for (let i = 0; i < 4; ++i) {
    lengthTracking[i] = MayNeedBigInt(lengthTracking, i + 1);
  }
  const evil = {
    valueOf: () => {
      rab.resize(2 * ctor.BYTES_PER_ELEMENT);
      return 3;
    }
  };
  assert.compareArray(ToNumbers(Array.prototype.slice.call(lengthTracking, 1, evil)), [
    2,
    undefined
  ]);
  assert.compareArray(ToNumbers(Array.prototype.slice.call(lengthTracking, 1, evil)), [
    2
  ]);
  assert.sameValue(rab.byteLength, 2 * ctor.BYTES_PER_ELEMENT);
}
