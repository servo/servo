// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%array%.prototype.indexof
description: >
  Array.p.indexOf behaves correctly for special float values on TypedArrays
  backed by resizable buffers.
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer, Array.prototype.includes]
---*/

for (let ctor of floatCtors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const lengthTracking = new ctor(rab);
  lengthTracking[0] = -Infinity;
  lengthTracking[1] = -Infinity;
  lengthTracking[2] = Infinity;
  lengthTracking[3] = Infinity;
  lengthTracking[4] = NaN;
  lengthTracking[5] = NaN;
  assert.sameValue(Array.prototype.indexOf.call(lengthTracking, -Infinity), 0);
  assert.sameValue(Array.prototype.indexOf.call(lengthTracking, Infinity), 2);
  // NaN is never found.
  assert.sameValue(Array.prototype.indexOf.call(lengthTracking, NaN), -1);
}
