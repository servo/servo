// Copyright (C) 2024 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Reflect.preventExtensions
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
  assert.sameValue(Reflect.preventExtensions(fixedLength), false);
  assert.sameValue(Reflect.preventExtensions(fixedLengthWithOffset), false);
  assert.sameValue(Reflect.preventExtensions(lengthTracking), false);
  assert.sameValue(Reflect.preventExtensions(lengthTrackingWithOffset), false);
}

for (let ctor of ctors) {
  const gsab = new SharedArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, { maxByteLength: 8 * ctor.BYTES_PER_ELEMENT });
  const fixedLength = new ctor(gsab, 0, 4);
  const fixedLengthWithOffset = new ctor(gsab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new ctor(gsab);
  const lengthTrackingWithOffset = new ctor(gsab, 2 * ctor.BYTES_PER_ELEMENT);

  // Fixed length TAs backed by GSABs can't shrink, and so are allowed.
  assert.sameValue(Reflect.preventExtensions(fixedLength), true);
  assert.sameValue(Reflect.preventExtensions(fixedLengthWithOffset), true);
  assert.sameValue(Reflect.preventExtensions(lengthTracking), false);
  assert.sameValue(Reflect.preventExtensions(lengthTrackingWithOffset), false);
}
