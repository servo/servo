// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.find
description: >
  Array.p.find behaves correctly when receiver is backed by resizable
  buffer that is shrunk mid-iteration
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

function ArrayFindHelper(ta, p) {
  return Array.prototype.find.call(ta, p);
}

// Orig. array: [0, 2, 4, 6]
//              [0, 2, 4, 6] << fixedLength
//                    [4, 6] << fixedLengthWithOffset
//              [0, 2, 4, 6, ...] << lengthTracking
//                    [4, 6, ...] << lengthTrackingWithOffset
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  const values = [];
  const resizeAfter = 2;
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  function CollectResize(n) {
    CollectValuesAndResize(n, values, rab, resizeAfter, resizeTo);
    return false;
  }
  assert.sameValue(ArrayFindHelper(fixedLength, CollectResize), undefined);
  assert.compareArray(values, [
    0,
    2,
    undefined,
    undefined
  ]);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const values = [];
  const resizeAfter = 1;
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  function CollectResize(n) {
    CollectValuesAndResize(n, values, rab, resizeAfter, resizeTo);
    return false;
  }
  assert.sameValue(ArrayFindHelper(fixedLengthWithOffset, CollectResize), undefined);
  assert.compareArray(values, [
    4,
    undefined
  ]);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab, 0);
  const values = [];
  const resizeAfter = 2;
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  function CollectResize(n) {
    CollectValuesAndResize(n, values, rab, resizeAfter, resizeTo);
    return false;
  }
  assert.sameValue(ArrayFindHelper(lengthTracking, CollectResize), undefined);
  assert.compareArray(values, [
    0,
    2,
    4,
    undefined
  ]);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  const values = [];
  const resizeAfter = 1;
  const resizeTo = 3 * ctor.BYTES_PER_ELEMENT;
  function CollectResize(n) {
    CollectValuesAndResize(n, values, rab, resizeAfter, resizeTo);
    return false;
  }
  assert.sameValue(ArrayFindHelper(lengthTrackingWithOffset, CollectResize), undefined);
  assert.compareArray(values, [
    4,
    undefined
  ]);
}
