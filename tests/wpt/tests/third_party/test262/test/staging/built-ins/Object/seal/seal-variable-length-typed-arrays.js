// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.seal
description: Can't seal variable length TypedArrays
features: [SharedArrayBuffer, ArrayBuffer, resizable-arraybuffer]
includes: [resizableArrayBufferUtils.js]
---*/

for (let ctor of ctors) {
  const rab = new ArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, { maxByteLength: 8 * ctor.BYTES_PER_ELEMENT });
  const fixedLength = new ctor(rab, 0, 4);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new ctor(rab);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);

  assert.notSameValue(fixedLength.length, 0);
  assert.notSameValue(fixedLengthWithOffset.length, 0);
  assert.notSameValue(lengthTracking.length, 0);
  assert.notSameValue(lengthTrackingWithOffset.length, 0);

  // "Fixed length" TAs backed by RABs can shrink then re-grow.
  assert.throws(TypeError, function() {
    Object.seal(fixedLength);
  });
  assert.throws(TypeError, function() {
    Object.seal(fixedLengthWithOffset);
  });
  assert.throws(TypeError, function() {
    Object.seal(lengthTracking);
  });
  assert.throws(TypeError, function() {
    Object.seal(lengthTrackingWithOffset);
  });
}

for (let ctor of ctors) {
  const rab = new ArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, { maxByteLength: 8 * ctor.BYTES_PER_ELEMENT });
  const rabZeroLength = new ArrayBuffer(0, { maxByteLength: 8 * ctor.BYTES_PER_ELEMENT });
  const fixedLength = new ctor(rab, 0, 0);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 0);
  const lengthTracking = new ctor(rabZeroLength);
  const lengthTrackingWithOffset = new ctor(rab, 4 * ctor.BYTES_PER_ELEMENT);

  assert.sameValue(fixedLength.length, 0);
  assert.sameValue(fixedLengthWithOffset.length, 0);
  assert.sameValue(lengthTracking.length, 0);
  assert.sameValue(lengthTrackingWithOffset.length, 0);

  // "Fixed length" TAs backed by RABs can shrink then re-grow.
  assert.throws(TypeError, function() {
    Object.seal(fixedLength);
  });
  assert.throws(TypeError, function() {
    Object.seal(fixedLengthWithOffset);
  });
  assert.throws(TypeError, function() {
    Object.seal(lengthTracking);
  });
  assert.throws(TypeError, function() {
    Object.seal(lengthTrackingWithOffset);
  });
}

for (let ctor of ctors) {
  const gsab = new SharedArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, { maxByteLength: 8 * ctor.BYTES_PER_ELEMENT });
  const fixedLength = new ctor(gsab, 0, 4);
  const fixedLengthWithOffset = new ctor(gsab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new ctor(gsab);
  const lengthTrackingWithOffset = new ctor(gsab, 2 * ctor.BYTES_PER_ELEMENT);

  assert.notSameValue(fixedLength.length, 0);
  assert.notSameValue(fixedLengthWithOffset.length, 0);
  assert.notSameValue(lengthTracking.length, 0);
  assert.notSameValue(lengthTrackingWithOffset.length, 0);

  // Fixed length TAs backed by GSABs can't shrink, and so are allowed. TypedArray
  // properties can't be redefined to non-configurable, so this still throws.
  assert.throws(TypeError, function() {
    Object.seal(fixedLength);
  });
  assert.throws(TypeError, function() {
    Object.seal(fixedLengthWithOffset);
  });
  assert.throws(TypeError, function() {
    Object.seal(lengthTracking);
  });
  assert.throws(TypeError, function() {
    Object.seal(lengthTrackingWithOffset);
  });
}

for (let ctor of ctors) {
  const gsab = new SharedArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, { maxByteLength: 8 * ctor.BYTES_PER_ELEMENT });
  const gsabZeroLength = new SharedArrayBuffer(0, { maxByteLength: 8 * ctor.BYTES_PER_ELEMENT });
  const fixedLength = new ctor(gsab, 0, 0);
  const fixedLengthWithOffset = new ctor(gsab, 2 * ctor.BYTES_PER_ELEMENT, 0);
  const lengthTracking = new ctor(gsabZeroLength);
  const lengthTrackingWithOffset = new ctor(gsab, 4 * ctor.BYTES_PER_ELEMENT);

  assert.sameValue(fixedLength.length, 0);
  assert.sameValue(fixedLengthWithOffset.length, 0);
  assert.sameValue(lengthTracking.length, 0);
  assert.sameValue(lengthTrackingWithOffset.length, 0);

  // Fixed zero-length TAs backed by GSABs can't shrink, and so are allowed. If
  // the TypedArray is zero length, there's no attempt to redefine TypedArray
  // properties.
  Object.seal(fixedLength);
  Object.seal(fixedLengthWithOffset);
  assert.throws(TypeError, function() {
    Object.seal(lengthTracking);
  });
  assert.throws(TypeError, function() {
    Object.seal(lengthTrackingWithOffset);
  });
}
