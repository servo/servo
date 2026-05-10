// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.freeze
description: >
  Object.freeze throws on non-0-length TypedArrays backed by resizable
  buffers
features: [resizable-arraybuffer]
includes: [resizableArrayBufferUtils.js]
---*/

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new ctor(rab, 0);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    Object.freeze(fixedLength);
  });
  assert.throws(TypeError, () => {
    Object.freeze(fixedLengthWithOffset);
  });
  assert.throws(TypeError, () => {
    Object.freeze(lengthTracking);
  });
  assert.throws(TypeError, () => {
    Object.freeze(lengthTrackingWithOffset);
  });
}
// Freezing zero-length TAs doesn't throw.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 0);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 0);
  const lengthTrackingWithOffset = new ctor(rab, 4 * ctor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    Object.freeze(fixedLength);
  });
  assert.throws(TypeError, () => {
    Object.freeze(fixedLengthWithOffset);
  });
  assert.throws(TypeError, () => {
    Object.freeze(lengthTrackingWithOffset);
  });
}
// If the buffer has been resized to make length-tracking TAs zero-length,
// freezing them also doesn't throw.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  rab.resize(2 * ctor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    Object.freeze(lengthTrackingWithOffset);
  });
  rab.resize(0 * ctor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    Object.freeze(lengthTracking);
  });
}
