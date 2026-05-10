// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
  Array.p.reduce behaves correctly when the resizable buffer is grown
  mid-iteration.
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
function ResizeMidIteration(acc, n) {
  // Returns true by default.
  return CollectValuesAndResize(n, values, rab, resizeAfter, resizeTo);
}

// Orig. array: [0, 2, 4, 6]
//              [0, 2, 4, 6] << fixedLength
//                    [4, 6] << fixedLengthWithOffset
//              [0, 2, 4, 6, ...] << lengthTracking
//                    [4, 6, ...] << lengthTrackingWithOffset

// Test for reduce.

for (let ctor of ctors) {
  values = [];
  rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  resizeAfter = 2;
  resizeTo = 5 * ctor.BYTES_PER_ELEMENT;
  Array.prototype.reduce.call(fixedLength, ResizeMidIteration, 'initial value');
  assert.compareArray(values, [
    0,
    2,
    4,
    6
  ]);
}
for (let ctor of ctors) {
  values = [];
  rab = CreateRabForTest(ctor);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  resizeAfter = 1;
  resizeTo = 5 * ctor.BYTES_PER_ELEMENT;
  Array.prototype.reduce.call(fixedLengthWithOffset, ResizeMidIteration, 'initial value');
  assert.compareArray(values, [
    4,
    6
  ]);
}
for (let ctor of ctors) {
  values = [];
  rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab, 0);
  resizeAfter = 2;
  resizeTo = 5 * ctor.BYTES_PER_ELEMENT;
  Array.prototype.reduce.call(lengthTracking, ResizeMidIteration, 'initial value');
  assert.compareArray(values, [
    0,
    2,
    4,
    6
  ]);
}
for (let ctor of ctors) {
  values = [];
  rab = CreateRabForTest(ctor);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  resizeAfter = 1;
  resizeTo = 5 * ctor.BYTES_PER_ELEMENT;
  Array.prototype.reduce.call(lengthTrackingWithOffset, ResizeMidIteration, 'initial value');
  assert.compareArray(values, [
    4,
    6
  ]);
}
