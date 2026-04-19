// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.preventExtensions
description: Can't preventExtensions variable length TypedArrays
features: [SharedArrayBuffer, ArrayBuffer, resizable-arraybuffer]
includes: [resizableArrayBufferUtils.js]
---*/

for (let ctor of ctors) {
  const rab = new ArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, { maxByteLength: 8 * ctor.BYTES_PER_ELEMENT });
  const fixedLength = new ctor(rab, 0, 4);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new ctor(rab);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);

  // "Fixed length" TAs backed by RABs can shrink then re-grow.
  assert.throws(TypeError, function() {
    Object.preventExtensions(fixedLength);
  });
  assert.throws(TypeError, function() {
    Object.preventExtensions(fixedLengthWithOffset);
  });
  assert.throws(TypeError, function() {
    Object.preventExtensions(lengthTracking);
  });
  assert.throws(TypeError, function() {
    Object.preventExtensions(lengthTrackingWithOffset);
  });
}

for (let ctor of ctors) {
  const gsab = new SharedArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, { maxByteLength: 8 * ctor.BYTES_PER_ELEMENT });
  const fixedLength = new ctor(gsab, 0, 4);
  const fixedLengthWithOffset = new ctor(gsab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new ctor(gsab);
  const lengthTrackingWithOffset = new ctor(gsab, 2 * ctor.BYTES_PER_ELEMENT);

  // Fixed length TAs backed by GSABs can't shrink, and so are allowed.
  Object.preventExtensions(fixedLength);
  Object.preventExtensions(fixedLengthWithOffset);
  assert.throws(TypeError, function() {
    Object.preventExtensions(lengthTracking);
  });
  assert.throws(TypeError, function() {
    Object.preventExtensions(lengthTrackingWithOffset);
  });
}
