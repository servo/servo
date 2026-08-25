// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.sort
description: >
  Array.p.sort behaves correctly on TypedArrays backed by resizable buffers and
  is shrunk by the comparison callback.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer, Array.prototype.includes]
---*/

function ResizeAndCompare(rab, resizeTo) {
  return (a, b) => {
    rab.resize(resizeTo);
    if (a < b) {
      return -1;
    }
    if (a > b) {
      return 1;
    }
    return 0;
  }
}

function WriteUnsortedData(taFull) {
  for (let i = 0; i < taFull.length; ++i) {
    taFull[i] = MayNeedBigInt(taFull, 10 - i);
  }
}

// Fixed length TA.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const resizeTo = 2 * ctor.BYTES_PER_ELEMENT;
  const fixedLength = new ctor(rab, 0, 4);
  const taFull = new ctor(rab, 0);
  WriteUnsortedData(taFull);
  Array.prototype.sort.call(fixedLength, ResizeAndCompare(rab, resizeTo));
  // The data is unchanged.
  assert.compareArray(ToNumbers(taFull), [
    10,
    9
  ]);
}

// Length-tracking TA.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const resizeTo = 2 * ctor.BYTES_PER_ELEMENT;
  const lengthTracking = new ctor(rab, 0);
  const taFull = new ctor(rab, 0);
  WriteUnsortedData(taFull);
  Array.prototype.sort.call(lengthTracking, ResizeAndCompare(rab, resizeTo));
  // The sort result is implementation defined, but it contains 2 elements out
  // of the 4 original ones.
  const newData = ToNumbers(taFull);
  assert.sameValue(newData.length, 2);
  assert([
    10,
    9,
    8,
    7
  ].includes(newData[0]));
  assert([
    10,
    9,
    8,
    7
  ].includes(newData[1]));
}
