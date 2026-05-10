// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
  Array.p.every behaves correctly when receiver is backed by resizable
  buffer that is grown mid-iteration
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

let values;
let rab;
let resizeAfter;
let resizeTo;
// Collects the view of the resizable array buffer rab into values, with an
// iteration during which, after resizeAfter steps, rab is resized to length
// resizeTo. To be called by a method of the view being collected.
// Note that rab, values, resizeAfter, and resizeTo may need to be reset
// before calling this.
function ResizeMidIteration(n) {
  // Returns true by default.
  return CollectValuesAndResize(n, values, rab, resizeAfter, resizeTo);
}

// Orig. array: [0, 2, 4, 6]
//              [0, 2, 4, 6] << fixedLength
//                    [4, 6] << fixedLengthWithOffset
//              [0, 2, 4, 6, ...] << lengthTracking
//                    [4, 6, ...] << lengthTrackingWithOffset
for (let ctor of ctors) {
  rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  values = [];
  resizeAfter = 2;
  resizeTo = 5 * ctor.BYTES_PER_ELEMENT;
  assert(Array.prototype.every.call(fixedLength, ResizeMidIteration));
  assert.compareArray(values, [
    0,
    2,
    4,
    6
  ]);
}
for (let ctor of ctors) {
  rab = CreateRabForTest(ctor);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  values = [];
  resizeAfter = 1;
  resizeTo = 5 * ctor.BYTES_PER_ELEMENT;
  assert(Array.prototype.every.call(fixedLengthWithOffset, ResizeMidIteration));
  assert.compareArray(values, [
    4,
    6
  ]);
}
for (let ctor of ctors) {
  rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab, 0);
  values = [];
  resizeAfter = 2;
  resizeTo = 5 * ctor.BYTES_PER_ELEMENT;
  assert(Array.prototype.every.call(lengthTracking, ResizeMidIteration));
  assert.compareArray(values, [
    0,
    2,
    4,
    6
  ]);
}
for (let ctor of ctors) {
  rab = CreateRabForTest(ctor);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  values = [];
  resizeAfter = 1;
  resizeTo = 5 * ctor.BYTES_PER_ELEMENT;
  assert(Array.prototype.every.call(lengthTrackingWithOffset, ResizeMidIteration));
  assert.compareArray(values, [
    4,
    6
  ]);
}
