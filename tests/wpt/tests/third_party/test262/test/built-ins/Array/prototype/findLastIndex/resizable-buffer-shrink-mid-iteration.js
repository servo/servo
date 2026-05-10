// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.findlastindex
description: >
  Array.p.findLastIndex behaves correctly when receiver is backed by resizable
  buffer that is shrunk mid-iteration
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
// Note that rab, values, resizeAfter, and resizeTo may need to be reset before
// calling this.
function ResizeMidIteration(n) {
  CollectValuesAndResize(n, values, rab, resizeAfter, resizeTo);
  return false;
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
  resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  assert.sameValue(Array.prototype.findLastIndex.call(fixedLength, ResizeMidIteration), -1);
  assert.compareArray(values, [
    6,
    4,
    undefined,
    undefined
  ]);
}
for (let ctor of ctors) {
  rab = CreateRabForTest(ctor);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  values = [];
  resizeAfter = 1;
  resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  assert.sameValue(Array.prototype.findLastIndex.call(fixedLengthWithOffset, ResizeMidIteration), -1);
  assert.compareArray(values, [
    6,
    undefined
  ]);
}
for (let ctor of ctors) {
  rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab, 0);
  values = [];
  resizeAfter = 2;
  resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  assert.sameValue(Array.prototype.findLastIndex.call(lengthTracking, ResizeMidIteration), -1);
  assert.compareArray(values, [
    6,
    4,
    2,
    0
  ]);
}
for (let ctor of ctors) {
  rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab, 0);
  values = [];
  resizeAfter = 1;
  resizeTo = 2 * ctor.BYTES_PER_ELEMENT;
  assert.sameValue(Array.prototype.findLastIndex.call(lengthTracking, ResizeMidIteration), -1);
  assert.compareArray(values, [
    6,
    undefined,
    2,
    0
  ]);
}
for (let ctor of ctors) {
  rab = CreateRabForTest(ctor);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  values = [];
  resizeAfter = 1;
  resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  assert.sameValue(Array.prototype.findLastIndex.call(lengthTrackingWithOffset, ResizeMidIteration), -1);
  assert.compareArray(values, [
    6,
    4
  ]);
}
