// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.sort
description: >
  TypedArray.p.sort behaves correctly on TypedArrays backed by resizable buffers
  which are grown by the comparison callback.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// Returns a function that resizes rab to size resizeTo and then compares its
// arguments. Such a result function is to be used as an argument to .sort.
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
  const resizeTo = 6 * ctor.BYTES_PER_ELEMENT;
  const fixedLength = new ctor(rab, 0, 4);
  const taFull = new ctor(rab, 0);
  WriteUnsortedData(taFull);
  fixedLength.sort(ResizeAndCompare(rab, resizeTo));
  // Growing doesn't affect the sorting.
  assert.compareArray(ToNumbers(taFull), [
    7,
    8,
    9,
    10,
    0,
    0
  ]);
}

// Length-tracking TA.
for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const resizeTo = 6 * ctor.BYTES_PER_ELEMENT;
  const lengthTracking = new ctor(rab, 0);
  const taFull = new ctor(rab, 0);
  WriteUnsortedData(taFull);
  lengthTracking.sort(ResizeAndCompare(rab, resizeTo));
  // Growing doesn't affect the sorting. Only the elements that were part of
  // the original TA are sorted.
  assert.compareArray(ToNumbers(taFull), [
    7,
    8,
    9,
    10,
    0,
    0
  ]);
}
