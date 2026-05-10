// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.some
description: >
  TypedArray.p.some behaves correctly on TypedArrays backed by resizable buffers
  which are grown mid-iteration.
features: [resizable-arraybuffer]
includes: [compareArray.js, resizableArrayBufferUtils.js]
---*/

let values;
let rab;
let resizeAfter;
let resizeTo;
// Below the argument array should be a view of the resizable array buffer rab.
// This array gets collected into values via reduce and CollectValuesAndResize.
// The latter performs an during which, after resizeAfter steps, rab is resized
// to length resizeTo. Note that rab, values, resizeAfter, and resizeTo may need
// to be reset before calling this.
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
  resizeTo = 5 * ctor.BYTES_PER_ELEMENT;
  assert(!fixedLength.some(ResizeMidIteration));
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
  assert(!fixedLengthWithOffset.some(ResizeMidIteration));
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
  assert(!lengthTracking.some(ResizeMidIteration));
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
  rab = rab;
  resizeAfter = 1;
  resizeTo = 5 * ctor.BYTES_PER_ELEMENT;
  assert(!lengthTrackingWithOffset.some(ResizeMidIteration));
  assert.compareArray(values, [
    4,
    6
  ]);
}
